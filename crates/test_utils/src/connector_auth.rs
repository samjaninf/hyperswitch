use std::{collections::HashMap, env};

use masking::Secret;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectorAuthentication {
    pub aci: Option<BodyKey>,
    #[cfg(not(feature = "payouts"))]
    pub adyen: Option<BodyKey>,
    #[cfg(feature = "payouts")]
    pub adyenplatform: Option<HeaderKey>,
    pub affirm: Option<HeaderKey>,
    #[cfg(feature = "payouts")]
    pub adyen: Option<SignatureKey>,
    #[cfg(not(feature = "payouts"))]
    pub adyen_uk: Option<BodyKey>,
    #[cfg(feature = "payouts")]
    pub adyen_uk: Option<SignatureKey>,
    pub airwallex: Option<BodyKey>,
    pub amazonpay: Option<HeaderKey>,
    pub archipel: Option<NoKey>,
    pub authipay: Option<SignatureKey>,
    pub authorizedotnet: Option<BodyKey>,
    pub bambora: Option<BodyKey>,
    pub bamboraapac: Option<HeaderKey>,
    pub bankofamerica: Option<SignatureKey>,
    pub barclaycard: Option<SignatureKey>,
    pub billwerk: Option<HeaderKey>,
    pub bitpay: Option<HeaderKey>,
    pub blackhawknetwork: Option<HeaderKey>,
    pub bluecode: Option<HeaderKey>,
    pub bluesnap: Option<BodyKey>,
    pub boku: Option<BodyKey>,
    pub breadpay: Option<BodyKey>,
    pub cashtocode: Option<BodyKey>,
    pub celero: Option<HeaderKey>,
    pub chargebee: Option<HeaderKey>,
    pub checkbook: Option<BodyKey>,
    pub checkout: Option<SignatureKey>,
    pub coinbase: Option<HeaderKey>,
    pub coingate: Option<HeaderKey>,
    pub cryptopay: Option<BodyKey>,
    pub cybersource: Option<SignatureKey>,
    pub datatrans: Option<HeaderKey>,
    pub deutschebank: Option<SignatureKey>,
    pub digitalvirgo: Option<HeaderKey>,
    pub dlocal: Option<SignatureKey>,
    #[cfg(feature = "dummy_connector")]
    pub dummyconnector: Option<HeaderKey>,
    pub dwolla: Option<HeaderKey>,
    pub ebanx: Option<HeaderKey>,
    pub elavon: Option<HeaderKey>,
    pub facilitapay: Option<BodyKey>,
    pub fiserv: Option<SignatureKey>,
    pub fiservemea: Option<HeaderKey>,
    pub fiuu: Option<HeaderKey>,
    pub flexiti: Option<HeaderKey>,
    pub forte: Option<MultiAuthKey>,
    pub getnet: Option<HeaderKey>,
    pub globalpay: Option<BodyKey>,
    pub globepay: Option<BodyKey>,
    pub gocardless: Option<HeaderKey>,
    pub gpayments: Option<HeaderKey>,
    pub helcim: Option<HeaderKey>,
    pub hipay: Option<HeaderKey>,
    pub hyperswitch_vault: Option<SignatureKey>,
    pub iatapay: Option<SignatureKey>,
    pub inespay: Option<HeaderKey>,
    pub itaubank: Option<MultiAuthKey>,
    pub jpmorgan: Option<BodyKey>,
    pub juspaythreedsserver: Option<HeaderKey>,
    pub katapult: Option<HeaderKey>,
    pub mifinity: Option<HeaderKey>,
    pub mollie: Option<BodyKey>,
    pub moneris: Option<SignatureKey>,
    pub mpgs: Option<HeaderKey>,
    pub multisafepay: Option<HeaderKey>,
    pub netcetera: Option<HeaderKey>,
    pub nexinets: Option<BodyKey>,
    pub nexixpay: Option<HeaderKey>,
    pub nomupay: Option<BodyKey>,
    pub noon: Option<SignatureKey>,
    pub nordea: Option<BodyKey>,
    pub novalnet: Option<HeaderKey>,
    pub nmi: Option<HeaderKey>,
    pub nuvei: Option<SignatureKey>,
    pub opayo: Option<HeaderKey>,
    pub opennode: Option<HeaderKey>,
    pub paybox: Option<HeaderKey>,
    pub payeezy: Option<SignatureKey>,
    pub payload: Option<CurrencyAuthKey>,
    pub payme: Option<BodyKey>,
    pub payone: Option<HeaderKey>,
    pub paypal: Option<BodyKey>,
    pub paystack: Option<HeaderKey>,
    pub paytm: Option<HeaderKey>,
    pub payu: Option<BodyKey>,
    pub phonepe: Option<HeaderKey>,
    pub placetopay: Option<BodyKey>,
    pub plaid: Option<BodyKey>,
    pub powertranz: Option<BodyKey>,
    pub prophetpay: Option<HeaderKey>,
    pub rapyd: Option<BodyKey>,
    pub razorpay: Option<BodyKey>,
    pub recurly: Option<HeaderKey>,
    pub redsys: Option<HeaderKey>,
    pub santander: Option<BodyKey>,
    pub shift4: Option<HeaderKey>,
    pub silverflow: Option<SignatureKey>,
    pub square: Option<BodyKey>,
    pub stax: Option<HeaderKey>,
    pub stripe: Option<HeaderKey>,
    pub stripebilling: Option<HeaderKey>,
    pub taxjar: Option<HeaderKey>,
    pub threedsecureio: Option<HeaderKey>,
    pub thunes: Option<HeaderKey>,
    pub tokenio: Option<HeaderKey>,
    pub stripe_au: Option<HeaderKey>,
    pub stripe_uk: Option<HeaderKey>,
    pub trustpay: Option<SignatureKey>,
    pub trustpayments: Option<HeaderKey>,
    pub tsys: Option<SignatureKey>,
    pub unified_authentication_service: Option<HeaderKey>,
    pub vgs: Option<SignatureKey>,
    pub volt: Option<HeaderKey>,
    pub wellsfargo: Option<HeaderKey>,
    // pub wellsfargopayout: Option<HeaderKey>,
    pub wise: Option<BodyKey>,
    pub worldpay: Option<BodyKey>,
    pub worldpayvantiv: Option<HeaderKey>,
    pub worldpayxml: Option<HeaderKey>,
    pub xendit: Option<HeaderKey>,
    pub worldline: Option<SignatureKey>,
    pub zen: Option<HeaderKey>,
    pub zsl: Option<BodyKey>,
    pub automation_configs: Option<AutomationConfigs>,
    pub users: Option<UsersConfigs>,
}

impl Default for ConnectorAuthentication {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ConnectorAuthentication {
    /// # Panics
    ///
    /// Will panic if `CONNECTOR_AUTH_FILE_PATH` env is not set
    #[allow(clippy::expect_used)]
    pub fn new() -> Self {
        // Do `export CONNECTOR_AUTH_FILE_PATH="/hyperswitch/crates/router/tests/connectors/sample_auth.toml"`
        // before running tests in shell
        let path = env::var("CONNECTOR_AUTH_FILE_PATH")
            .expect("Connector authentication file path not set");
        toml::from_str(
            &std::fs::read_to_string(path).expect("connector authentication config file not found"),
        )
        .expect("Failed to read connector authentication config file")
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectorAuthenticationMap(HashMap<String, ConnectorAuthType>);

impl Default for ConnectorAuthenticationMap {
    fn default() -> Self {
        Self::new()
    }
}

// This is a temporary solution to avoid rust compiler from complaining about unused function
#[allow(dead_code)]
impl ConnectorAuthenticationMap {
    pub fn inner(&self) -> &HashMap<String, ConnectorAuthType> {
        &self.0
    }

    /// # Panics
    ///
    /// Will panic if `CONNECTOR_AUTH_FILE_PATH` env  is not set
    #[allow(clippy::expect_used)]
    pub fn new() -> Self {
        // Do `export CONNECTOR_AUTH_FILE_PATH="/hyperswitch/crates/router/tests/connectors/sample_auth.toml"`
        // before running tests in shell
        let path = env::var("CONNECTOR_AUTH_FILE_PATH")
            .expect("connector authentication file path not set");

        // Read the file contents to a JsonString
        let contents =
            &std::fs::read_to_string(path).expect("Failed to read connector authentication file");

        // Deserialize the JsonString to a HashMap
        let auth_config: HashMap<String, toml::Value> =
            toml::from_str(contents).expect("Failed to deserialize TOML file");

        // auth_config contains the data in below given format:
        // {
        //  "connector_name": Table(
        //      {
        //          "api_key": String(
        //                 "API_Key",
        //          ),
        //          "api_secret": String(
        //              "Secret key",
        //          ),
        //          "key1": String(
        //                  "key1",
        //          ),
        //          "key2": String(
        //              "key2",
        //          ),
        //      },
        //  ),
        // "connector_name": Table(
        //  ...
        // }

        // auth_map refines and extracts required information
        let auth_map = auth_config
            .into_iter()
            .map(|(connector_name, config)| {
                let auth_type = match config {
                    toml::Value::Table(mut table) => {
                        if let Some(auth_key_map_value) = table.remove("auth_key_map") {
                            // This is a CurrencyAuthKey
                            if let toml::Value::Table(auth_key_map_table) = auth_key_map_value {
                                let mut parsed_auth_map = HashMap::new();
                                for (currency, val) in auth_key_map_table {
                                    if let Ok(currency_enum) =
                                        currency.parse::<common_enums::Currency>()
                                    {
                                        parsed_auth_map
                                            .insert(currency_enum, Secret::new(val.to_string()));
                                    }
                                }
                                ConnectorAuthType::CurrencyAuthKey {
                                    auth_key_map: parsed_auth_map,
                                }
                            } else {
                                ConnectorAuthType::NoKey
                            }
                        } else {
                            match (
                                table.get("api_key"),
                                table.get("key1"),
                                table.get("api_secret"),
                                table.get("key2"),
                            ) {
                                (Some(api_key), None, None, None) => ConnectorAuthType::HeaderKey {
                                    api_key: Secret::new(
                                        api_key.as_str().unwrap_or_default().to_string(),
                                    ),
                                },
                                (Some(api_key), Some(key1), None, None) => {
                                    ConnectorAuthType::BodyKey {
                                        api_key: Secret::new(
                                            api_key.as_str().unwrap_or_default().to_string(),
                                        ),
                                        key1: Secret::new(
                                            key1.as_str().unwrap_or_default().to_string(),
                                        ),
                                    }
                                }
                                (Some(api_key), Some(key1), Some(api_secret), None) => {
                                    ConnectorAuthType::SignatureKey {
                                        api_key: Secret::new(
                                            api_key.as_str().unwrap_or_default().to_string(),
                                        ),
                                        key1: Secret::new(
                                            key1.as_str().unwrap_or_default().to_string(),
                                        ),
                                        api_secret: Secret::new(
                                            api_secret.as_str().unwrap_or_default().to_string(),
                                        ),
                                    }
                                }
                                (Some(api_key), Some(key1), Some(api_secret), Some(key2)) => {
                                    ConnectorAuthType::MultiAuthKey {
                                        api_key: Secret::new(
                                            api_key.as_str().unwrap_or_default().to_string(),
                                        ),
                                        key1: Secret::new(
                                            key1.as_str().unwrap_or_default().to_string(),
                                        ),
                                        api_secret: Secret::new(
                                            api_secret.as_str().unwrap_or_default().to_string(),
                                        ),
                                        key2: Secret::new(
                                            key2.as_str().unwrap_or_default().to_string(),
                                        ),
                                    }
                                }
                                _ => ConnectorAuthType::NoKey,
                            }
                        }
                    }
                    _ => ConnectorAuthType::NoKey,
                };
                (connector_name, auth_type)
            })
            .collect();
        Self(auth_map)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeaderKey {
    pub api_key: Secret<String>,
}

impl From<HeaderKey> for ConnectorAuthType {
    fn from(key: HeaderKey) -> Self {
        Self::HeaderKey {
            api_key: key.api_key,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BodyKey {
    pub api_key: Secret<String>,
    pub key1: Secret<String>,
}

impl From<BodyKey> for ConnectorAuthType {
    fn from(key: BodyKey) -> Self {
        Self::BodyKey {
            api_key: key.api_key,
            key1: key.key1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignatureKey {
    pub api_key: Secret<String>,
    pub key1: Secret<String>,
    pub api_secret: Secret<String>,
}

impl From<SignatureKey> for ConnectorAuthType {
    fn from(key: SignatureKey) -> Self {
        Self::SignatureKey {
            api_key: key.api_key,
            key1: key.key1,
            api_secret: key.api_secret,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiAuthKey {
    pub api_key: Secret<String>,
    pub key1: Secret<String>,
    pub api_secret: Secret<String>,
    pub key2: Secret<String>,
}

impl From<MultiAuthKey> for ConnectorAuthType {
    fn from(key: MultiAuthKey) -> Self {
        Self::MultiAuthKey {
            api_key: key.api_key,
            key1: key.key1,
            api_secret: key.api_secret,
            key2: key.key2,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrencyAuthKey {
    pub auth_key_map: HashMap<String, toml::Value>,
}

impl From<CurrencyAuthKey> for ConnectorAuthType {
    fn from(key: CurrencyAuthKey) -> Self {
        let mut auth_map = HashMap::new();
        for (currency, auth_data) in key.auth_key_map {
            if let Ok(currency_enum) = currency.parse::<common_enums::Currency>() {
                auth_map.insert(currency_enum, Secret::new(auth_data.to_string()));
            }
        }
        Self::CurrencyAuthKey {
            auth_key_map: auth_map,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoKey {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutomationConfigs {
    pub hs_base_url: Option<String>,
    pub hs_api_key: Option<String>,
    pub hs_api_keys: Option<String>,
    pub hs_webhook_url: Option<String>,
    pub hs_test_env: Option<String>,
    pub hs_test_browser: Option<String>,
    pub chrome_profile_path: Option<String>,
    pub firefox_profile_path: Option<String>,
    pub pypl_email: Option<String>,
    pub pypl_pass: Option<String>,
    pub gmail_email: Option<String>,
    pub gmail_pass: Option<String>,
    pub clearpay_email: Option<String>,
    pub clearpay_pass: Option<String>,
    pub configs_url: Option<String>,
    pub stripe_pub_key: Option<String>,
    pub testcases_path: Option<String>,
    pub bluesnap_gateway_merchant_id: Option<String>,
    pub globalpay_gateway_merchant_id: Option<String>,
    pub authorizedotnet_gateway_merchant_id: Option<String>,
    pub run_minimum_steps: Option<bool>,
    pub airwallex_merchant_name: Option<String>,
    pub adyen_bancontact_username: Option<String>,
    pub adyen_bancontact_pass: Option<String>,
}

#[derive(Default, Debug, Clone, serde::Deserialize)]
#[serde(tag = "auth_type")]
pub enum ConnectorAuthType {
    HeaderKey {
        api_key: Secret<String>,
    },
    BodyKey {
        api_key: Secret<String>,
        key1: Secret<String>,
    },
    SignatureKey {
        api_key: Secret<String>,
        key1: Secret<String>,
        api_secret: Secret<String>,
    },
    MultiAuthKey {
        api_key: Secret<String>,
        key1: Secret<String>,
        api_secret: Secret<String>,
        key2: Secret<String>,
    },
    CurrencyAuthKey {
        auth_key_map: HashMap<common_enums::Currency, Secret<String>>,
    },
    #[default]
    NoKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsersConfigs {
    pub user_email: String,
    pub user_password: String,
    pub wrong_password: String,
    pub user_base_email_for_signup: String,
    pub user_domain_for_signup: String,
}
