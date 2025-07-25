use common_utils::consts::TENANT_HEADER;
use futures::StreamExt;
use router_env::{
    logger,
    tracing::{field::Empty, Instrument},
};

use crate::{headers, routes::metrics};

/// Middleware to include request ID in response header.
pub struct RequestId;

impl<S, B> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for RequestId
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = RequestIdMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(RequestIdMiddleware { service }))
    }
}

pub struct RequestIdMiddleware<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<actix_web::dev::ServiceRequest> for RequestIdMiddleware<S>
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = futures::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        let old_x_request_id = req.headers().get("x-request-id").cloned();
        let mut req = req;
        let request_id_fut = req.extract::<router_env::tracing_actix_web::RequestId>();
        let response_fut = self.service.call(req);

        Box::pin(
            async move {
                let request_id = request_id_fut.await?;
                let request_id = request_id.as_hyphenated().to_string();
                if let Some(upstream_request_id) = old_x_request_id {
                    router_env::logger::info!(?upstream_request_id);
                }
                let mut response = response_fut.await?;
                response.headers_mut().append(
                    http::header::HeaderName::from_static("x-request-id"),
                    http::HeaderValue::from_str(&request_id)?,
                );

                Ok(response)
            }
            .in_current_span(),
        )
    }
}

/// Middleware for attaching default response headers. Headers with the same key already set in a
/// response will not be overwritten.
pub fn default_response_headers() -> actix_web::middleware::DefaultHeaders {
    use actix_web::http::header;

    let default_headers_middleware = actix_web::middleware::DefaultHeaders::new();

    #[cfg(feature = "vergen")]
    let default_headers_middleware =
        default_headers_middleware.add(("x-hyperswitch-version", router_env::git_tag!()));

    default_headers_middleware
        // Max age of 1 year in seconds, equal to `60 * 60 * 24 * 365` seconds.
        .add((header::STRICT_TRANSPORT_SECURITY, "max-age=31536000"))
        .add((header::VIA, "HyperSwitch"))
}

/// Middleware to build a TOP level domain span for each request.
pub struct LogSpanInitializer;

impl<S, B> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for LogSpanInitializer
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = LogSpanInitializerMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(LogSpanInitializerMiddleware { service }))
    }
}

pub struct LogSpanInitializerMiddleware<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<actix_web::dev::ServiceRequest>
    for LogSpanInitializerMiddleware<S>
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = futures::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    // TODO: have a common source of truth for the list of top level fields
    // /crates/router_env/src/logger/storage.rs also has a list of fields  called PERSISTENT_KEYS
    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        let tenant_id = req
            .headers()
            .get(TENANT_HEADER)
            .and_then(|i| i.to_str().ok())
            .map(|s| s.to_owned());
        let response_fut = self.service.call(req);
        let tenant_id_clone = tenant_id.clone();
        Box::pin(
            async move {
                if let Some(tenant) = tenant_id_clone {
                    router_env::tracing::Span::current().record("tenant_id", tenant);
                }
                let response = response_fut.await;
                router_env::tracing::Span::current().record("golden_log_line", true);
                response
            }
            .instrument(
                router_env::tracing::info_span!(
                    "ROOT_SPAN",
                    payment_id = Empty,
                    merchant_id = Empty,
                    connector_name = Empty,
                    payment_method = Empty,
                    status_code = Empty,
                    flow = "UNKNOWN",
                    golden_log_line = Empty,
                    tenant_id = &tenant_id
                )
                .or_current(),
            ),
        )
    }
}

fn get_request_details_from_value(json_value: &serde_json::Value, parent_key: &str) -> String {
    match json_value {
        serde_json::Value::Null => format!("{parent_key}: null"),
        serde_json::Value::Bool(b) => format!("{parent_key}: {b}"),
        serde_json::Value::Number(num) => format!("{}: {}", parent_key, num.to_string().len()),
        serde_json::Value::String(s) => format!("{}: {}", parent_key, s.len()),
        serde_json::Value::Array(arr) => {
            let mut result = String::new();
            for (index, value) in arr.iter().enumerate() {
                let child_key = format!("{parent_key}[{index}]");
                result.push_str(&get_request_details_from_value(value, &child_key));
                if index < arr.len() - 1 {
                    result.push_str(", ");
                }
            }
            result
        }
        serde_json::Value::Object(obj) => {
            let mut result = String::new();
            for (index, (key, value)) in obj.iter().enumerate() {
                let child_key = format!("{parent_key}[{key}]");
                result.push_str(&get_request_details_from_value(value, &child_key));
                if index < obj.len() - 1 {
                    result.push_str(", ");
                }
            }
            result
        }
    }
}

/// Middleware for Logging request_details of HTTP 400 Bad Requests
pub struct Http400RequestDetailsLogger;

impl<S: 'static, B> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest>
    for Http400RequestDetailsLogger
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = Http400RequestDetailsLoggerMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(Http400RequestDetailsLoggerMiddleware {
            service: std::rc::Rc::new(service),
        }))
    }
}

pub struct Http400RequestDetailsLoggerMiddleware<S> {
    service: std::rc::Rc<S>,
}

impl<S, B> actix_web::dev::Service<actix_web::dev::ServiceRequest>
    for Http400RequestDetailsLoggerMiddleware<S>
where
    S: actix_web::dev::Service<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = futures::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, mut req: actix_web::dev::ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let request_id_fut = req.extract::<router_env::tracing_actix_web::RequestId>();
        Box::pin(async move {
            let (http_req, payload) = req.into_parts();
            let result_payload: Vec<Result<bytes::Bytes, actix_web::error::PayloadError>> =
                payload.collect().await;
            let payload = result_payload
                .into_iter()
                .collect::<Result<Vec<bytes::Bytes>, actix_web::error::PayloadError>>()?;
            let bytes = payload.clone().concat().to_vec();
            let bytes_length = bytes.len();
            // we are creating h1 payload manually from bytes, currently there's no way to create http2 payload with actix
            let (_, mut new_payload) = actix_http::h1::Payload::create(true);
            new_payload.unread_data(bytes.to_vec().clone().into());
            let new_req = actix_web::dev::ServiceRequest::from_parts(http_req, new_payload.into());

            let content_length_header = new_req
                .headers()
                .get(headers::CONTENT_LENGTH)
                .map(ToOwned::to_owned);
            let response_fut = svc.call(new_req);
            let response = response_fut.await?;
            // Log the request_details when we receive 400 status from the application
            if response.status() == 400 {
                let request_id = request_id_fut.await?.as_hyphenated().to_string();
                let content_length_header_string = content_length_header
                    .map(|content_length_header| {
                        content_length_header.to_str().map(ToOwned::to_owned)
                    })
                    .transpose()
                    .inspect_err(|error| {
                        logger::warn!("Could not convert content length to string {error:?}");
                    })
                    .ok()
                    .flatten();

                logger::info!("Content length from header: {content_length_header_string:?}, Bytes length: {bytes_length}");

                if !bytes.is_empty() {
                    let value_result: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_slice(&bytes);
                    match value_result {
                        Ok(value) => {
                            logger::info!(
                                "request_id: {request_id}, request_details: {}",
                                get_request_details_from_value(&value, "")
                            );
                        }
                        Err(err) => {
                            logger::warn!("error while parsing the request in json value: {err}");
                        }
                    }
                } else {
                    logger::info!("request_id: {request_id}, request_details: Empty Body");
                }
            }
            Ok(response)
        })
    }
}

/// Middleware for Adding Accept-Language header based on query params
pub struct AddAcceptLanguageHeader;

impl<S: 'static, B> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest>
    for AddAcceptLanguageHeader
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = AddAcceptLanguageHeaderMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(AddAcceptLanguageHeaderMiddleware {
            service: std::rc::Rc::new(service),
        }))
    }
}

pub struct AddAcceptLanguageHeaderMiddleware<S> {
    service: std::rc::Rc<S>,
}

impl<S, B> actix_web::dev::Service<actix_web::dev::ServiceRequest>
    for AddAcceptLanguageHeaderMiddleware<S>
where
    S: actix_web::dev::Service<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = futures::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, mut req: actix_web::dev::ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            #[derive(serde::Deserialize)]
            struct LocaleQueryParam {
                locale: Option<String>,
            }
            let query_params = req.query_string();
            let locale_param =
                serde_qs::from_str::<LocaleQueryParam>(query_params).map_err(|error| {
                    actix_web::error::ErrorBadRequest(format!(
                        "Could not convert query params to locale query parmas: {error:?}",
                    ))
                })?;
            let accept_language_header = req.headers().get(http::header::ACCEPT_LANGUAGE);
            if let Some(locale) = locale_param.locale {
                req.headers_mut().insert(
                    http::header::ACCEPT_LANGUAGE,
                    http::HeaderValue::from_str(&locale)?,
                );
            } else if accept_language_header.is_none() {
                req.headers_mut().insert(
                    http::header::ACCEPT_LANGUAGE,
                    http::HeaderValue::from_static("en"),
                );
            }
            let response_fut = svc.call(req);
            let response = response_fut.await?;
            Ok(response)
        })
    }
}

/// Middleware for recording request-response metrics
pub struct RequestResponseMetrics;

impl<S: 'static, B> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest>
    for RequestResponseMetrics
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = RequestResponseMetricsMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(RequestResponseMetricsMiddleware {
            service: std::rc::Rc::new(service),
        }))
    }
}

pub struct RequestResponseMetricsMiddleware<S> {
    service: std::rc::Rc<S>,
}

impl<S, B> actix_web::dev::Service<actix_web::dev::ServiceRequest>
    for RequestResponseMetricsMiddleware<S>
where
    S: actix_web::dev::Service<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = futures::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        use std::borrow::Cow;

        let svc = self.service.clone();

        let request_path = req
            .match_pattern()
            .map(Cow::<'static, str>::from)
            .unwrap_or_else(|| "UNKNOWN".into());
        let request_method = Cow::<'static, str>::from(req.method().as_str().to_owned());

        Box::pin(async move {
            let mut attributes =
                router_env::metric_attributes!(("path", request_path), ("method", request_method))
                    .to_vec();

            let response_fut = svc.call(req);

            metrics::REQUESTS_RECEIVED.add(1, &attributes);

            let (response_result, request_duration) =
                common_utils::metrics::utils::time_future(response_fut).await;
            let response = response_result?;

            attributes.extend_from_slice(router_env::metric_attributes!((
                "status_code",
                i64::from(response.status().as_u16())
            )));

            metrics::REQUEST_TIME.record(request_duration.as_secs_f64(), &attributes);

            Ok(response)
        })
    }
}
