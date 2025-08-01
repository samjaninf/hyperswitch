/// Refunds - Create
///
/// Creates a refund against an already processed payment. In case of some processors, you can even opt to refund only a partial amount multiple times until the original charge amount has been refunded
#[utoipa::path(
    post,
    path = "/refunds",
    request_body(
        content = RefundRequest,
        examples(
            (
                "Create an instant refund to refund the whole amount" = (
                    value = json!({
                        "payment_id": "{{payment_id}}",
                        "refund_type": "instant"
                      })
                )
            ),
            (
                "Create an instant refund to refund partial amount" = (
                    value = json!({
                        "payment_id": "{{payment_id}}",
                        "refund_type": "instant",
                        "amount": 654
                      })
                )
            ),
            (
                "Create an instant refund with reason" = (
                    value = json!({
                        "payment_id": "{{payment_id}}",
                        "refund_type": "instant",
                        "amount": 6540,
                        "reason": "Customer returned product"
                      })
                )
            ),
        )
    ),
    responses(
        (status = 200, description = "Refund created", body = RefundResponse),
        (status = 400, description = "Missing Mandatory fields", body = GenericErrorResponseOpenApi)
    ),
    tag = "Refunds",
    operation_id = "Create a Refund",
    security(("api_key" = []))
)]
#[cfg(feature = "v1")]
pub async fn refunds_create() {}

/// Refunds - Retrieve
///
/// Retrieves a Refund. This may be used to get the status of a previously initiated refund
#[utoipa::path(
    get,
    path = "/refunds/{refund_id}",
    params(
        ("refund_id" = String, Path, description = "The identifier for refund")
    ),
    responses(
        (status = 200, description = "Refund retrieved", body = RefundResponse),
        (status = 404, description = "Refund does not exist in our records")
    ),
    tag = "Refunds",
    operation_id = "Retrieve a Refund",
    security(("api_key" = []))
)]
#[cfg(feature = "v1")]
pub async fn refunds_retrieve() {}

/// Refunds - Retrieve (POST)
///
/// To retrieve the properties of a Refund. This may be used to get the status of a previously initiated payment or next action for an ongoing payment
#[utoipa::path(
    get,
    path = "/refunds/sync",
    responses(
        (status = 200, description = "Refund retrieved", body = RefundResponse),
        (status = 404, description = "Refund does not exist in our records")
    ),
    tag = "Refunds",
    operation_id = "Retrieve a Refund",
    security(("api_key" = []))
)]
pub async fn refunds_retrieve_with_body() {}

/// Refunds - Update
///
/// Updates the properties of a Refund object. This API can be used to attach a reason for the refund or metadata fields
#[utoipa::path(
    post,
    path = "/refunds/{refund_id}",
    params(
        ("refund_id" = String, Path, description = "The identifier for refund")
    ),
    request_body(
        content = RefundUpdateRequest,
        examples(
            (
                "Update refund reason" = (
                    value = json!({
                        "reason": "Paid by mistake"
                      })
                )
            ),
        )
    ),
    responses(
        (status = 200, description = "Refund updated", body = RefundResponse),
        (status = 400, description = "Missing Mandatory fields", body = GenericErrorResponseOpenApi)
    ),
    tag = "Refunds",
    operation_id = "Update a Refund",
    security(("api_key" = []))
)]
#[cfg(feature = "v1")]
pub async fn refunds_update() {}

/// Refunds - List
///
/// Lists all the refunds associated with the merchant, or for a specific payment if payment_id is provided
#[utoipa::path(
    post,
    path = "/refunds/list",
    request_body=RefundListRequest,
    responses(
        (status = 200, description = "List of refunds", body = RefundListResponse),
    ),
    tag = "Refunds",
    operation_id = "List all Refunds",
    security(("api_key" = []))
)]
#[cfg(feature = "v1")]
pub fn refunds_list() {}

/// Refunds - List For the Given profiles
///
/// Lists all the refunds associated with the merchant or a payment_id if payment_id is not provided
#[utoipa::path(
    post,
    path = "/refunds/profile/list",
    request_body=RefundListRequest,
    responses(
        (status = 200, description = "List of refunds", body = RefundListResponse),
    ),
    tag = "Refunds",
    operation_id = "List all Refunds for the given Profiles",
    security(("api_key" = []))
)]
pub fn refunds_list_profile() {}

/// Refunds - Filter
///
/// To list the refunds filters associated with list of connectors, currencies and payment statuses
#[utoipa::path(
    post,
    path = "/refunds/filter",
    request_body=TimeRange,
    responses(
        (status = 200, description = "List of filters", body = RefundListMetaData),
    ),
    tag = "Refunds",
    operation_id = "List all filters for Refunds",
    security(("api_key" = []))
)]
pub async fn refunds_filter_list() {}

/// Refunds - Create
///
/// Creates a refund against an already processed payment. In case of some processors, you can even opt to refund only a partial amount multiple times until the original charge amount has been refunded
#[utoipa::path(
    post,
    path = "/v2/refunds",
    request_body(
        content = RefundsCreateRequest,
        examples(
            (
                "Create an instant refund to refund the whole amount" = (
                    value = json!({
                        "payment_id": "{{payment_id}}",
                        "merchant_reference_id": "ref_123",
                        "refund_type": "instant"
                      })
                )
            ),
            (
                "Create an instant refund to refund partial amount" = (
                    value = json!({
                        "payment_id": "{{payment_id}}",
                        "merchant_reference_id": "ref_123",
                        "refund_type": "instant",
                        "amount": 654
                      })
                )
            ),
            (
                "Create an instant refund with reason" = (
                    value = json!({
                        "payment_id": "{{payment_id}}",
                        "merchant_reference_id": "ref_123",
                        "refund_type": "instant",
                        "amount": 6540,
                        "reason": "Customer returned product"
                      })
                )
            ),
        )
    ),
    responses(
        (status = 200, description = "Refund created", body = RefundResponse),
        (status = 400, description = "Missing Mandatory fields", body = GenericErrorResponseOpenApi)
    ),
    tag = "Refunds",
    operation_id = "Create a Refund",
    security(("api_key" = []))
)]
#[cfg(feature = "v2")]
pub async fn refunds_create() {}

/// Refunds - Metadata Update
///
/// Updates the properties of a Refund object. This API can be used to attach a reason for the refund or metadata fields
#[utoipa::path(
    put,
    path = "/v2/refunds/{id}/update-metadata",
    params(
        ("id" = String, Path, description = "The identifier for refund")
    ),
    request_body(
        content = RefundMetadataUpdateRequest,
        examples(
            (
                "Update refund reason" = (
                    value = json!({
                        "reason": "Paid by mistake"
                      })
                )
            )
        )
    ),
    responses(
        (status = 200, description = "Refund updated", body = RefundResponse),
        (status = 400, description = "Missing Mandatory fields", body = GenericErrorResponseOpenApi)
    ),
    tag = "Refunds",
    operation_id = "Update Refund Metadata and Reason",
    security(("api_key" = []))
)]
#[cfg(feature = "v2")]
pub async fn refunds_metadata_update() {}

/// Refunds - Retrieve
///
/// Retrieves a Refund. This may be used to get the status of a previously initiated refund
#[utoipa::path(
    get,
    path = "/v2/refunds/{id}",
    params(
        ("id" = String, Path, description = "The identifier for refund")
    ),
    responses(
        (status = 200, description = "Refund retrieved", body = RefundResponse),
        (status = 404, description = "Refund does not exist in our records")
    ),
    tag = "Refunds",
    operation_id = "Retrieve a Refund",
    security(("api_key" = []))
)]
#[cfg(feature = "v2")]
pub async fn refunds_retrieve() {}

/// Refunds - List
///
/// To list the refunds associated with a payment_id or with the merchant, if payment_id is not provided
#[utoipa::path(
    post,
    path = "/v2/refunds/list",
    request_body=RefundListRequest,
    responses(
        (status = 200, description = "List of refunds", body = RefundListResponse),
    ),
    tag = "Refunds",
    operation_id = "List all Refunds",
    security(("api_key" = []))
)]
#[cfg(feature = "v2")]
pub fn refunds_list() {}
