use cards::CardNumber;
use common_enums::enums;
use common_utils::{
    errors::CustomResult,
    ext_traits::{OptionExt, ValueExt},
    pii::{self},
    request::Method,
};
use error_stack::ResultExt;
use hyperswitch_domain_models::{
    payment_method_data::{
        BankDebitData, BankRedirectData, BankTransferData, Card, CardRedirectData, GiftCardData,
        PayLaterData, PaymentMethodData, VoucherData, WalletData,
    },
    router_data::{ConnectorAuthType, ErrorResponse, RouterData},
    router_flow_types::refunds::{Execute, RSync},
    router_request_types::{BrowserInformation, ResponseId},
    router_response_types::{PaymentsResponseData, RedirectForm, RefundsResponseData},
    types,
};
use hyperswitch_interfaces::{
    api,
    consts::{NO_ERROR_CODE, NO_ERROR_MESSAGE},
    errors,
};
use masking::{ExposeInterface, PeekInterface, Secret};
use ring::digest;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{
    types::{RefundsResponseRouterData, ResponseRouterData},
    utils::{
        self, BrowserInformationData, CardData, ForeignTryFrom, PaymentsAuthorizeRequestData,
        RouterData as _,
    },
};

#[derive(Debug, Serialize)]
pub struct ZenRouterData<T> {
    pub amount: String,
    pub router_data: T,
}

impl<T> TryFrom<(&api::CurrencyUnit, enums::Currency, i64, T)> for ZenRouterData<T> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        (currency_unit, currency, amount, item): (&api::CurrencyUnit, enums::Currency, i64, T),
    ) -> Result<Self, Self::Error> {
        let amount = utils::get_amount_as_string(currency_unit, amount, currency)?;
        Ok(Self {
            amount,
            router_data: item,
        })
    }
}

// Auth Struct
pub struct ZenAuthType {
    pub(super) api_key: Secret<String>,
}

impl TryFrom<&ConnectorAuthType> for ZenAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &ConnectorAuthType) -> Result<Self, Self::Error> {
        if let ConnectorAuthType::HeaderKey { api_key } = auth_type {
            Ok(Self {
                api_key: api_key.to_owned(),
            })
        } else {
            Err(errors::ConnectorError::FailedToObtainAuthType.into())
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiRequest {
    merchant_transaction_id: String,
    payment_channel: ZenPaymentChannels,
    amount: String,
    currency: enums::Currency,
    payment_specific_data: ZenPaymentSpecificData,
    customer: ZenCustomerDetails,
    custom_ipn_url: String,
    items: Vec<ZenItemObject>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ZenPaymentsRequest {
    ApiRequest(Box<ApiRequest>),
    CheckoutRequest(Box<CheckoutRequest>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutRequest {
    amount: String,
    currency: enums::Currency,
    custom_ipn_url: String,
    items: Vec<ZenItemObject>,
    merchant_transaction_id: String,
    signature: Option<Secret<String>>,
    specified_payment_channel: ZenPaymentChannels,
    terminal_uuid: Secret<String>,
    url_redirect: String,
}

#[derive(Clone, Debug, Display, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(clippy::enum_variant_names)]
pub enum ZenPaymentChannels {
    PclCard,
    PclGooglepay,
    PclApplepay,
    PclBoacompraBoleto,
    PclBoacompraEfecty,
    PclBoacompraMultibanco,
    PclBoacompraPagoefectivo,
    PclBoacompraPix,
    PclBoacompraPse,
    PclBoacompraRedcompra,
    PclBoacompraRedpagos,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenCustomerDetails {
    email: pii::Email,
    ip: Secret<String, pii::IpAddress>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ZenPaymentSpecificData {
    ZenOnetimePayment(Box<ZenPaymentData>),
    ZenGeneralPayment(ZenGeneralPaymentData),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenPaymentData {
    browser_details: ZenBrowserDetails,
    #[serde(rename = "type")]
    payment_type: ZenPaymentTypes,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<Secret<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    card: Option<ZenCardDetails>,
    descriptor: String,
    return_verify_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenGeneralPaymentData {
    #[serde(rename = "type")]
    payment_type: ZenPaymentTypes,
    return_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenBrowserDetails {
    color_depth: String,
    java_enabled: bool,
    lang: String,
    screen_height: String,
    screen_width: String,
    timezone: String,
    accept_header: String,
    window_size: String,
    user_agent: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ZenPaymentTypes {
    Onetime,
    General,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenCardDetails {
    number: CardNumber,
    expiry_date: Secret<String>,
    cvv: Secret<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenItemObject {
    name: String,
    price: String,
    quantity: u16,
    line_amount_total: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionObject {
    pub apple_pay: Option<WalletSessionData>,
    pub google_pay: Option<WalletSessionData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletSessionData {
    pub terminal_uuid: Option<Secret<String>>,
    pub pay_wall_secret: Option<Secret<String>>,
}

impl TryFrom<(&ZenRouterData<&types::PaymentsAuthorizeRouterData>, &Card)> for ZenPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        value: (&ZenRouterData<&types::PaymentsAuthorizeRouterData>, &Card),
    ) -> Result<Self, Self::Error> {
        let (item, ccard) = value;
        let browser_info = item.router_data.request.get_browser_info()?;
        let ip = browser_info.get_ip_address()?;
        let browser_details = get_browser_details(&browser_info)?;
        let amount = item.amount.to_owned();
        let payment_specific_data =
            ZenPaymentSpecificData::ZenOnetimePayment(Box::new(ZenPaymentData {
                browser_details,
                //Connector Specific for cards
                payment_type: ZenPaymentTypes::Onetime,
                token: None,
                card: Some(ZenCardDetails {
                    number: ccard.card_number.clone(),
                    expiry_date: ccard
                        .get_card_expiry_month_year_2_digit_with_delimiter("".to_owned())?,
                    cvv: ccard.card_cvc.clone(),
                }),
                descriptor: item
                    .router_data
                    .get_description()?
                    .chars()
                    .take(24)
                    .collect(),
                return_verify_url: item.router_data.request.router_return_url.clone(),
            }));
        Ok(Self::ApiRequest(Box::new(ApiRequest {
            merchant_transaction_id: item.router_data.connector_request_reference_id.clone(),
            payment_channel: ZenPaymentChannels::PclCard,
            currency: item.router_data.request.currency,
            payment_specific_data,
            customer: get_customer(item.router_data, ip)?,
            custom_ipn_url: item.router_data.request.get_webhook_url()?,
            items: get_item_object(item.router_data)?,
            amount,
        })))
    }
}

impl
    TryFrom<(
        &ZenRouterData<&types::PaymentsAuthorizeRouterData>,
        &VoucherData,
    )> for ZenPaymentsRequest
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        value: (
            &ZenRouterData<&types::PaymentsAuthorizeRouterData>,
            &VoucherData,
        ),
    ) -> Result<Self, Self::Error> {
        let (item, voucher_data) = value;
        let browser_info = item.router_data.request.get_browser_info()?;
        let ip = browser_info.get_ip_address()?;
        let amount = item.amount.to_owned();
        let payment_specific_data =
            ZenPaymentSpecificData::ZenGeneralPayment(ZenGeneralPaymentData {
                //Connector Specific for Latam Methods
                payment_type: ZenPaymentTypes::General,
                return_url: item.router_data.request.get_router_return_url()?,
            });
        let payment_channel = match voucher_data {
            VoucherData::Boleto { .. } => ZenPaymentChannels::PclBoacompraBoleto,
            VoucherData::Efecty => ZenPaymentChannels::PclBoacompraEfecty,
            VoucherData::PagoEfectivo => ZenPaymentChannels::PclBoacompraPagoefectivo,
            VoucherData::RedCompra => ZenPaymentChannels::PclBoacompraRedcompra,
            VoucherData::RedPagos => ZenPaymentChannels::PclBoacompraRedpagos,
            VoucherData::Oxxo
            | VoucherData::Alfamart { .. }
            | VoucherData::Indomaret { .. }
            | VoucherData::SevenEleven { .. }
            | VoucherData::Lawson { .. }
            | VoucherData::MiniStop { .. }
            | VoucherData::FamilyMart { .. }
            | VoucherData::Seicomart { .. }
            | VoucherData::PayEasy { .. } => Err(errors::ConnectorError::NotImplemented(
                utils::get_unimplemented_payment_method_error_message("Zen"),
            ))?,
        };
        Ok(Self::ApiRequest(Box::new(ApiRequest {
            merchant_transaction_id: item.router_data.connector_request_reference_id.clone(),
            payment_channel,
            currency: item.router_data.request.currency,
            payment_specific_data,
            customer: get_customer(item.router_data, ip)?,
            custom_ipn_url: item.router_data.request.get_webhook_url()?,
            items: get_item_object(item.router_data)?,
            amount,
        })))
    }
}

impl
    TryFrom<(
        &ZenRouterData<&types::PaymentsAuthorizeRouterData>,
        &Box<BankTransferData>,
    )> for ZenPaymentsRequest
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        value: (
            &ZenRouterData<&types::PaymentsAuthorizeRouterData>,
            &Box<BankTransferData>,
        ),
    ) -> Result<Self, Self::Error> {
        let (item, bank_transfer_data) = value;
        let browser_info = item.router_data.request.get_browser_info()?;
        let ip = browser_info.get_ip_address()?;
        let amount = item.amount.to_owned();
        let payment_specific_data =
            ZenPaymentSpecificData::ZenGeneralPayment(ZenGeneralPaymentData {
                //Connector Specific for Latam Methods
                payment_type: ZenPaymentTypes::General,
                return_url: item.router_data.request.get_router_return_url()?,
            });
        let payment_channel = match **bank_transfer_data {
            BankTransferData::MultibancoBankTransfer { .. } => {
                ZenPaymentChannels::PclBoacompraMultibanco
            }
            BankTransferData::Pix { .. } => ZenPaymentChannels::PclBoacompraPix,
            BankTransferData::Pse { .. } => ZenPaymentChannels::PclBoacompraPse,
            BankTransferData::SepaBankTransfer { .. }
            | BankTransferData::AchBankTransfer { .. }
            | BankTransferData::BacsBankTransfer { .. }
            | BankTransferData::PermataBankTransfer { .. }
            | BankTransferData::BcaBankTransfer { .. }
            | BankTransferData::BniVaBankTransfer { .. }
            | BankTransferData::BriVaBankTransfer { .. }
            | BankTransferData::CimbVaBankTransfer { .. }
            | BankTransferData::DanamonVaBankTransfer { .. }
            | BankTransferData::LocalBankTransfer { .. }
            | BankTransferData::InstantBankTransfer {}
            | BankTransferData::InstantBankTransferFinland { .. }
            | BankTransferData::InstantBankTransferPoland { .. }
            | BankTransferData::IndonesianBankTransfer { .. }
            | BankTransferData::MandiriVaBankTransfer { .. } => {
                Err(errors::ConnectorError::NotImplemented(
                    utils::get_unimplemented_payment_method_error_message("Zen"),
                ))?
            }
        };
        Ok(Self::ApiRequest(Box::new(ApiRequest {
            merchant_transaction_id: item.router_data.connector_request_reference_id.clone(),
            payment_channel,
            currency: item.router_data.request.currency,
            payment_specific_data,
            customer: get_customer(item.router_data, ip)?,
            custom_ipn_url: item.router_data.request.get_webhook_url()?,
            items: get_item_object(item.router_data)?,
            amount,
        })))
    }
}

/*
impl TryFrom<(&types::PaymentsAuthorizeRouterData, &GooglePayWalletData)> for ZenPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        (item, gpay_pay_redirect_data): (&types::PaymentsAuthorizeRouterData, &GooglePayWalletData),
    ) -> Result<Self, Self::Error> {
        let amount = utils::to_currency_base_unit(item.request.amount, item.request.currency)?;
        let browser_info = item.request.get_browser_info()?;
        let browser_details = get_browser_details(&browser_info)?;
        let ip = browser_info.get_ip_address()?;
        let payment_specific_data = ZenPaymentData {
            browser_details,
            //Connector Specific for wallet
            payment_type: ZenPaymentTypes::ExternalPaymentToken,
            token: Some(Secret::new(
                gpay_pay_redirect_data.tokenization_data.token.clone(),
            )),
            card: None,
            descriptor: item.get_description()?.chars().take(24).collect(),
            return_verify_url: item.request.router_return_url.clone(),
        };
        Ok(Self::ApiRequest(Box::new(ApiRequest {
            merchant_transaction_id: item.attempt_id.clone(),
            payment_channel: ZenPaymentChannels::PclGooglepay,
            currency: item.request.currency,
            payment_specific_data,
            customer: get_customer(item, ip)?,
            custom_ipn_url: item.request.get_webhook_url()?,
            items: get_item_object(item, amount.clone())?,
            amount,
        })))
    }
}
*/
/*
impl
    TryFrom<(
        &types::PaymentsAuthorizeRouterData,
        &Box<ApplePayRedirectData>,
    )> for ZenPaymentsRequest
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        (item, _apple_pay_redirect_data): (
            &types::PaymentsAuthorizeRouterData,
            &Box<ApplePayRedirectData>,
        ),
    ) -> Result<Self, Self::Error> {
        let amount = utils::to_currency_base_unit(item.request.amount, item.request.currency)?;
        let connector_meta = item.get_connector_meta()?;
        let session: SessionObject = connector_meta
            .parse_value("SessionObject")
            .change_context(errors::ConnectorError::RequestEncodingFailed)?;
        let applepay_session_data = session
            .apple_pay
            .ok_or(errors::ConnectorError::RequestEncodingFailed)?;
        let terminal_uuid = applepay_session_data
            .terminal_uuid
            .clone()
            .ok_or(errors::ConnectorError::RequestEncodingFailed)?;
        let mut checkout_request = CheckoutRequest {
            merchant_transaction_id: item.attempt_id.clone(),
            specified_payment_channel: ZenPaymentChannels::PclApplepay,
            currency: item.request.currency,
            custom_ipn_url: item.request.get_webhook_url()?,
            items: get_item_object(item, amount.clone())?,
            amount,
            terminal_uuid: Secret::new(terminal_uuid),
            signature: None,
            url_redirect: item.request.get_return_url()?,
        };
        checkout_request.signature = Some(get_checkout_signature(
            &checkout_request,
            &applepay_session_data,
        )?);
        Ok(Self::CheckoutRequest(Box::new(checkout_request)))
    }
}
*/

impl
    TryFrom<(
        &ZenRouterData<&types::PaymentsAuthorizeRouterData>,
        &WalletData,
    )> for ZenPaymentsRequest
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        (item, wallet_data): (
            &ZenRouterData<&types::PaymentsAuthorizeRouterData>,
            &WalletData,
        ),
    ) -> Result<Self, Self::Error> {
        let amount = item.amount.to_owned();
        let connector_meta = item.router_data.get_connector_meta()?;
        let session: SessionObject = connector_meta
            .parse_value("SessionObject")
            .change_context(errors::ConnectorError::RequestEncodingFailed)?;
        let (specified_payment_channel, session_data) = match wallet_data {
            WalletData::ApplePayRedirect(_) => (
                ZenPaymentChannels::PclApplepay,
                session
                    .apple_pay
                    .ok_or(errors::ConnectorError::InvalidWalletToken {
                        wallet_name: "Apple Pay".to_string(),
                    })?,
            ),
            WalletData::GooglePayRedirect(_) => (
                ZenPaymentChannels::PclGooglepay,
                session
                    .google_pay
                    .ok_or(errors::ConnectorError::InvalidWalletToken {
                        wallet_name: "Google Pay".to_string(),
                    })?,
            ),
            WalletData::WeChatPayRedirect(_)
            | WalletData::PaypalRedirect(_)
            | WalletData::ApplePay(_)
            | WalletData::GooglePay(_)
            | WalletData::BluecodeRedirect {}
            | WalletData::AliPayQr(_)
            | WalletData::AliPayRedirect(_)
            | WalletData::AliPayHkRedirect(_)
            | WalletData::AmazonPayRedirect(_)
            | WalletData::Paysera(_)
            | WalletData::Skrill(_)
            | WalletData::MomoRedirect(_)
            | WalletData::KakaoPayRedirect(_)
            | WalletData::GoPayRedirect(_)
            | WalletData::GcashRedirect(_)
            | WalletData::ApplePayThirdPartySdk(_)
            | WalletData::DanaRedirect {}
            | WalletData::GooglePayThirdPartySdk(_)
            | WalletData::MbWayRedirect(_)
            | WalletData::MobilePayRedirect(_)
            | WalletData::PaypalSdk(_)
            | WalletData::Paze(_)
            | WalletData::SamsungPay(_)
            | WalletData::TwintRedirect {}
            | WalletData::VippsRedirect {}
            | WalletData::TouchNGoRedirect(_)
            | WalletData::CashappQr(_)
            | WalletData::SwishQr(_)
            | WalletData::WeChatPayQr(_)
            | WalletData::Mifinity(_)
            | WalletData::RevolutPay(_) => Err(errors::ConnectorError::NotImplemented(
                utils::get_unimplemented_payment_method_error_message("Zen"),
            ))?,
        };
        let terminal_uuid = session_data
            .terminal_uuid
            .clone()
            .ok_or(errors::ConnectorError::RequestEncodingFailed)?
            .expose();
        let mut checkout_request = CheckoutRequest {
            merchant_transaction_id: item.router_data.connector_request_reference_id.clone(),
            specified_payment_channel,
            currency: item.router_data.request.currency,
            custom_ipn_url: item.router_data.request.get_webhook_url()?,
            items: get_item_object(item.router_data)?,
            amount,
            terminal_uuid: Secret::new(terminal_uuid),
            signature: None,
            url_redirect: item.router_data.request.get_router_return_url()?,
        };
        checkout_request.signature =
            Some(get_checkout_signature(&checkout_request, &session_data)?);
        Ok(Self::CheckoutRequest(Box::new(checkout_request)))
    }
}

fn get_checkout_signature(
    checkout_request: &CheckoutRequest,
    session: &WalletSessionData,
) -> Result<Secret<String>, error_stack::Report<errors::ConnectorError>> {
    let pay_wall_secret = session
        .pay_wall_secret
        .clone()
        .ok_or(errors::ConnectorError::RequestEncodingFailed)?;
    let mut signature_data = get_signature_data(checkout_request)?;
    signature_data.push_str(&pay_wall_secret.expose());
    let payload_digest = digest::digest(&digest::SHA256, signature_data.as_bytes());
    let mut signature = hex::encode(payload_digest);
    signature.push_str(";sha256");
    Ok(Secret::new(signature))
}

/// Fields should be in alphabetical order
fn get_signature_data(
    checkout_request: &CheckoutRequest,
) -> Result<String, errors::ConnectorError> {
    let specified_payment_channel = match checkout_request.specified_payment_channel {
        ZenPaymentChannels::PclCard => "pcl_card",
        ZenPaymentChannels::PclGooglepay => "pcl_googlepay",
        ZenPaymentChannels::PclApplepay => "pcl_applepay",
        ZenPaymentChannels::PclBoacompraBoleto => "pcl_boacompra_boleto",
        ZenPaymentChannels::PclBoacompraEfecty => "pcl_boacompra_efecty",
        ZenPaymentChannels::PclBoacompraMultibanco => "pcl_boacompra_multibanco",
        ZenPaymentChannels::PclBoacompraPagoefectivo => "pcl_boacompra_pagoefectivo",
        ZenPaymentChannels::PclBoacompraPix => "pcl_boacompra_pix",
        ZenPaymentChannels::PclBoacompraPse => "pcl_boacompra_pse",
        ZenPaymentChannels::PclBoacompraRedcompra => "pcl_boacompra_redcompra",
        ZenPaymentChannels::PclBoacompraRedpagos => "pcl_boacompra_redpagos",
    };
    let mut signature_data = vec![
        format!("amount={}", checkout_request.amount),
        format!("currency={}", checkout_request.currency),
        format!("customipnurl={}", checkout_request.custom_ipn_url),
    ];
    for index in 0..checkout_request.items.len() {
        let prefix = format!("items[{index}].");
        let checkout_request_items = checkout_request
            .items
            .get(index)
            .ok_or(errors::ConnectorError::RequestEncodingFailed)?;
        signature_data.push(format!(
            "{prefix}lineamounttotal={}",
            checkout_request_items.line_amount_total
        ));
        signature_data.push(format!("{prefix}name={}", checkout_request_items.name));
        signature_data.push(format!("{prefix}price={}", checkout_request_items.price));
        signature_data.push(format!(
            "{prefix}quantity={}",
            checkout_request_items.quantity
        ));
    }
    signature_data.push(format!(
        "merchanttransactionid={}",
        checkout_request.merchant_transaction_id
    ));
    signature_data.push(format!(
        "specifiedpaymentchannel={specified_payment_channel}"
    ));
    signature_data.push(format!(
        "terminaluuid={}",
        checkout_request.terminal_uuid.peek()
    ));
    signature_data.push(format!("urlredirect={}", checkout_request.url_redirect));
    let signature = signature_data.join("&");
    Ok(signature.to_lowercase())
}

fn get_customer(
    item: &types::PaymentsAuthorizeRouterData,
    ip: Secret<String, pii::IpAddress>,
) -> Result<ZenCustomerDetails, error_stack::Report<errors::ConnectorError>> {
    Ok(ZenCustomerDetails {
        email: item.request.get_email()?,
        ip,
    })
}

fn get_item_object(
    item: &types::PaymentsAuthorizeRouterData,
) -> Result<Vec<ZenItemObject>, error_stack::Report<errors::ConnectorError>> {
    let order_details = item.request.get_order_details()?;

    order_details
        .iter()
        .map(|data| {
            Ok(ZenItemObject {
                name: data.product_name.clone(),
                quantity: data.quantity,
                price: utils::to_currency_base_unit_with_zero_decimal_check(
                    data.amount.get_amount_as_i64(), // This should be changed to MinorUnit when we implement amount conversion for this connector. Additionally, the function get_amount_as_i64() should be avoided in the future.
                    item.request.currency,
                )?,
                line_amount_total: (f64::from(data.quantity)
                    * utils::to_currency_base_unit_asf64(
                        data.amount.get_amount_as_i64(), // This should be changed to MinorUnit when we implement amount conversion for this connector. Additionally, the function get_amount_as_i64() should be avoided in the future.
                        item.request.currency,
                    )?)
                .to_string(),
            })
        })
        .collect::<Result<_, _>>()
}

fn get_browser_details(
    browser_info: &BrowserInformation,
) -> CustomResult<ZenBrowserDetails, errors::ConnectorError> {
    let screen_height = browser_info
        .screen_height
        .get_required_value("screen_height")
        .change_context(errors::ConnectorError::MissingRequiredField {
            field_name: "screen_height",
        })?;

    let screen_width = browser_info
        .screen_width
        .get_required_value("screen_width")
        .change_context(errors::ConnectorError::MissingRequiredField {
            field_name: "screen_width",
        })?;

    let window_size = match (screen_height, screen_width) {
        (250, 400) => "01",
        (390, 400) => "02",
        (500, 600) => "03",
        (600, 400) => "04",
        _ => "05",
    }
    .to_string();

    Ok(ZenBrowserDetails {
        color_depth: browser_info.get_color_depth()?.to_string(),
        java_enabled: browser_info.get_java_enabled()?,
        lang: browser_info.get_language()?,
        screen_height: screen_height.to_string(),
        screen_width: screen_width.to_string(),
        timezone: browser_info.get_time_zone()?.to_string(),
        accept_header: browser_info.get_accept_header()?,
        user_agent: browser_info.get_user_agent()?,
        window_size,
    })
}

impl TryFrom<&ZenRouterData<&types::PaymentsAuthorizeRouterData>> for ZenPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: &ZenRouterData<&types::PaymentsAuthorizeRouterData>,
    ) -> Result<Self, Self::Error> {
        match &item.router_data.request.payment_method_data {
            PaymentMethodData::Card(card) => Self::try_from((item, card)),
            PaymentMethodData::Wallet(wallet_data) => Self::try_from((item, wallet_data)),
            PaymentMethodData::Voucher(voucher_data) => Self::try_from((item, voucher_data)),
            PaymentMethodData::BankTransfer(bank_transfer_data) => {
                Self::try_from((item, bank_transfer_data))
            }
            PaymentMethodData::BankRedirect(bank_redirect_data) => {
                Self::try_from(bank_redirect_data)
            }
            PaymentMethodData::PayLater(paylater_data) => Self::try_from(paylater_data),
            PaymentMethodData::BankDebit(bank_debit_data) => Self::try_from(bank_debit_data),
            PaymentMethodData::CardRedirect(car_redirect_data) => Self::try_from(car_redirect_data),
            PaymentMethodData::GiftCard(gift_card_data) => Self::try_from(gift_card_data.as_ref()),
            PaymentMethodData::Crypto(_)
            | PaymentMethodData::MandatePayment
            | PaymentMethodData::Reward
            | PaymentMethodData::RealTimePayment(_)
            | PaymentMethodData::MobilePayment(_)
            | PaymentMethodData::Upi(_)
            | PaymentMethodData::OpenBanking(_)
            | PaymentMethodData::CardToken(_)
            | PaymentMethodData::NetworkToken(_)
            | PaymentMethodData::CardDetailsForNetworkTransactionId(_) => {
                Err(errors::ConnectorError::NotImplemented(
                    utils::get_unimplemented_payment_method_error_message("Zen"),
                ))?
            }
        }
    }
}

impl TryFrom<&BankRedirectData> for ZenPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(value: &BankRedirectData) -> Result<Self, Self::Error> {
        match value {
            BankRedirectData::Ideal { .. }
            | BankRedirectData::Sofort { .. }
            | BankRedirectData::BancontactCard { .. }
            | BankRedirectData::Blik { .. }
            | BankRedirectData::Trustly { .. }
            | BankRedirectData::Eft { .. }
            | BankRedirectData::Eps { .. }
            | BankRedirectData::Giropay { .. }
            | BankRedirectData::Przelewy24 { .. }
            | BankRedirectData::Bizum {}
            | BankRedirectData::Interac { .. }
            | BankRedirectData::OnlineBankingCzechRepublic { .. }
            | BankRedirectData::OnlineBankingFinland { .. }
            | BankRedirectData::OnlineBankingPoland { .. }
            | BankRedirectData::OnlineBankingSlovakia { .. }
            | BankRedirectData::OpenBankingUk { .. }
            | BankRedirectData::OnlineBankingFpx { .. }
            | BankRedirectData::OnlineBankingThailand { .. }
            | BankRedirectData::LocalBankRedirect {} => {
                Err(errors::ConnectorError::NotImplemented(
                    utils::get_unimplemented_payment_method_error_message("Zen"),
                )
                .into())
            }
        }
    }
}

impl TryFrom<&PayLaterData> for ZenPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(value: &PayLaterData) -> Result<Self, Self::Error> {
        match value {
            PayLaterData::KlarnaRedirect { .. }
            | PayLaterData::KlarnaSdk { .. }
            | PayLaterData::AffirmRedirect {}
            | PayLaterData::AfterpayClearpayRedirect { .. }
            | PayLaterData::PayBrightRedirect {}
            | PayLaterData::WalleyRedirect {}
            | PayLaterData::AlmaRedirect {}
            | PayLaterData::FlexitiRedirect {}
            | PayLaterData::AtomeRedirect {}
            | PayLaterData::BreadpayRedirect {} => Err(errors::ConnectorError::NotImplemented(
                utils::get_unimplemented_payment_method_error_message("Zen"),
            )
            .into()),
        }
    }
}

impl TryFrom<&BankDebitData> for ZenPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(value: &BankDebitData) -> Result<Self, Self::Error> {
        match value {
            BankDebitData::AchBankDebit { .. }
            | BankDebitData::SepaBankDebit { .. }
            | BankDebitData::BecsBankDebit { .. }
            | BankDebitData::BacsBankDebit { .. } => Err(errors::ConnectorError::NotImplemented(
                utils::get_unimplemented_payment_method_error_message("Zen"),
            )
            .into()),
        }
    }
}

impl TryFrom<&CardRedirectData> for ZenPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(value: &CardRedirectData) -> Result<Self, Self::Error> {
        match value {
            CardRedirectData::Knet {}
            | CardRedirectData::Benefit {}
            | CardRedirectData::MomoAtm {}
            | CardRedirectData::CardRedirect {} => Err(errors::ConnectorError::NotImplemented(
                utils::get_unimplemented_payment_method_error_message("Zen"),
            )
            .into()),
        }
    }
}

impl TryFrom<&GiftCardData> for ZenPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(value: &GiftCardData) -> Result<Self, Self::Error> {
        match value {
            GiftCardData::PaySafeCard {} | GiftCardData::Givex(_) => {
                Err(errors::ConnectorError::NotImplemented(
                    utils::get_unimplemented_payment_method_error_message("Zen"),
                )
                .into())
            }
        }
    }
}

// PaymentsResponse
#[derive(Debug, Default, Deserialize, Clone, strum::Display, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ZenPaymentStatus {
    Authorized,
    Accepted,
    #[default]
    Pending,
    Rejected,
    Canceled,
}

impl ForeignTryFrom<(ZenPaymentStatus, Option<ZenActions>)> for enums::AttemptStatus {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn foreign_try_from(item: (ZenPaymentStatus, Option<ZenActions>)) -> Result<Self, Self::Error> {
        let (item_txn_status, item_action_status) = item;
        Ok(match item_txn_status {
            // Payment has been authorized at connector end, They will send webhook when it gets accepted
            ZenPaymentStatus::Authorized => Self::Pending,
            ZenPaymentStatus::Accepted => Self::Charged,
            ZenPaymentStatus::Pending => {
                item_action_status.map_or(Self::Pending, |action| match action {
                    ZenActions::Redirect => Self::AuthenticationPending,
                })
            }
            ZenPaymentStatus::Rejected => Self::Failure,
            ZenPaymentStatus::Canceled => Self::Voided,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    status: ZenPaymentStatus,
    id: String,
    // merchant_transaction_id: Option<String>,
    merchant_action: Option<ZenMerchantAction>,
    reject_code: Option<String>,
    reject_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ZenPaymentsResponse {
    ApiResponse(ApiResponse),
    CheckoutResponse(CheckoutResponse),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutResponse {
    redirect_url: url::Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenMerchantAction {
    action: ZenActions,
    data: ZenMerchantActionData,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ZenActions {
    Redirect,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenMerchantActionData {
    redirect_url: url::Url,
}

impl<F, T> TryFrom<ResponseRouterData<F, ZenPaymentsResponse, T, PaymentsResponseData>>
    for RouterData<F, T, PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: ResponseRouterData<F, ZenPaymentsResponse, T, PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        match item.response {
            ZenPaymentsResponse::ApiResponse(response) => Self::try_from(ResponseRouterData {
                response,
                data: item.data,
                http_code: item.http_code,
            }),
            ZenPaymentsResponse::CheckoutResponse(response) => Self::try_from(ResponseRouterData {
                response,
                data: item.data,
                http_code: item.http_code,
            }),
        }
    }
}

fn get_zen_response(
    response: ApiResponse,
    status_code: u16,
) -> CustomResult<
    (
        enums::AttemptStatus,
        Option<ErrorResponse>,
        PaymentsResponseData,
    ),
    errors::ConnectorError,
> {
    let redirection_data_action = response.merchant_action.map(|merchant_action| {
        (
            RedirectForm::from((merchant_action.data.redirect_url, Method::Get)),
            merchant_action.action,
        )
    });
    let (redirection_data, action) = match redirection_data_action {
        Some((redirect_form, action)) => (Some(redirect_form), Some(action)),
        None => (None, None),
    };
    let status = enums::AttemptStatus::foreign_try_from((response.status, action))?;
    let error = if utils::is_payment_failure(status) {
        Some(ErrorResponse {
            code: response
                .reject_code
                .unwrap_or_else(|| NO_ERROR_CODE.to_string()),
            message: response
                .reject_reason
                .clone()
                .unwrap_or_else(|| NO_ERROR_MESSAGE.to_string()),
            reason: response.reject_reason,
            status_code,
            attempt_status: Some(status),
            connector_transaction_id: Some(response.id.clone()),
            network_advice_code: None,
            network_decline_code: None,
            network_error_message: None,
        })
    } else {
        None
    };
    let payment_response_data = PaymentsResponseData::TransactionResponse {
        resource_id: ResponseId::ConnectorTransactionId(response.id.clone()),
        redirection_data: Box::new(redirection_data),
        mandate_reference: Box::new(None),
        connector_metadata: None,
        network_txn_id: None,
        connector_response_reference_id: None,
        incremental_authorization_allowed: None,
        charges: None,
    };
    Ok((status, error, payment_response_data))
}

impl<F, T> TryFrom<ResponseRouterData<F, ApiResponse, T, PaymentsResponseData>>
    for RouterData<F, T, PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        value: ResponseRouterData<F, ApiResponse, T, PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        let (status, error, payment_response_data) =
            get_zen_response(value.response.clone(), value.http_code)?;

        Ok(Self {
            status,
            response: error.map_or_else(|| Ok(payment_response_data), Err),
            ..value.data
        })
    }
}

impl<F, T> TryFrom<ResponseRouterData<F, CheckoutResponse, T, PaymentsResponseData>>
    for RouterData<F, T, PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        value: ResponseRouterData<F, CheckoutResponse, T, PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        let redirection_data = Some(RedirectForm::from((
            value.response.redirect_url,
            Method::Get,
        )));
        Ok(Self {
            status: enums::AttemptStatus::AuthenticationPending,
            response: Ok(PaymentsResponseData::TransactionResponse {
                resource_id: ResponseId::NoResponseId,
                redirection_data: Box::new(redirection_data),
                mandate_reference: Box::new(None),
                connector_metadata: None,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
                charges: None,
            }),
            ..value.data
        })
    }
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenRefundRequest {
    amount: String,
    transaction_id: String,
    currency: enums::Currency,
    merchant_transaction_id: String,
}

impl<F> TryFrom<&ZenRouterData<&types::RefundsRouterData<F>>> for ZenRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ZenRouterData<&types::RefundsRouterData<F>>) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: item.amount.to_owned(),
            transaction_id: item.router_data.request.connector_transaction_id.clone(),
            currency: item.router_data.request.currency,
            merchant_transaction_id: item.router_data.request.refund_id.clone(),
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RefundStatus {
    Authorized,
    Accepted,
    #[default]
    Pending,
    Rejected,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Accepted => Self::Success,
            RefundStatus::Pending | RefundStatus::Authorized => Self::Pending,
            RefundStatus::Rejected => Self::Failure,
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundResponse {
    id: String,
    status: RefundStatus,
    reject_code: Option<String>,
    reject_reason: Option<String>,
}

impl TryFrom<RefundsResponseRouterData<Execute, RefundResponse>>
    for types::RefundsRouterData<Execute>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        let (error, refund_response_data) = get_zen_refund_response(item.response, item.http_code)?;
        Ok(Self {
            response: error.map_or_else(|| Ok(refund_response_data), Err),
            ..item.data
        })
    }
}

fn get_zen_refund_response(
    response: RefundResponse,
    status_code: u16,
) -> CustomResult<(Option<ErrorResponse>, RefundsResponseData), errors::ConnectorError> {
    let refund_status = enums::RefundStatus::from(response.status);
    let error = if utils::is_refund_failure(refund_status) {
        Some(ErrorResponse {
            code: response
                .reject_code
                .unwrap_or_else(|| NO_ERROR_CODE.to_string()),
            message: response
                .reject_reason
                .clone()
                .unwrap_or_else(|| NO_ERROR_MESSAGE.to_string()),
            reason: response.reject_reason,
            status_code,
            attempt_status: None,
            connector_transaction_id: Some(response.id.clone()),
            network_advice_code: None,
            network_decline_code: None,
            network_error_message: None,
        })
    } else {
        None
    };
    let refund_response_data = RefundsResponseData {
        connector_refund_id: response.id,
        refund_status,
    };
    Ok((error, refund_response_data))
}

impl TryFrom<RefundsResponseRouterData<RSync, RefundResponse>> for types::RefundsRouterData<RSync> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<RSync, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        let refund_status = enums::RefundStatus::from(item.response.status);
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id,
                refund_status,
            }),
            ..item.data
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenWebhookBody {
    #[serde(rename = "transactionId")]
    pub id: String,
    pub merchant_transaction_id: String,
    pub amount: String,
    pub currency: String,
    pub status: ZenPaymentStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZenWebhookSignature {
    pub hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenWebhookObjectReference {
    #[serde(rename = "type")]
    pub transaction_type: ZenWebhookTxnType,
    pub transaction_id: String,
    pub merchant_transaction_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenWebhookEventType {
    #[serde(rename = "type")]
    pub transaction_type: ZenWebhookTxnType,
    pub transaction_id: String,
    pub status: ZenPaymentStatus,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZenWebhookTxnType {
    TrtPurchase,
    TrtRefund,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ZenErrorResponse {
    pub error: Option<ZenErrorBody>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ZenErrorBody {
    pub message: String,
    pub code: String,
}
