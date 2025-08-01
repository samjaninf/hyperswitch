use crate::{dirval, dssa::types::AnalysisErrorType, enums as global_enums, frontend::dir};

pub trait IntoDirValue {
    fn into_dir_value(self) -> Result<dir::DirValue, AnalysisErrorType>;
}
impl IntoDirValue for (global_enums::PaymentMethodType, global_enums::PaymentMethod) {
    fn into_dir_value(self) -> Result<dir::DirValue, AnalysisErrorType> {
        match self.0 {
            global_enums::PaymentMethodType::Credit => Ok(dirval!(CardType = Credit)),
            global_enums::PaymentMethodType::Debit => Ok(dirval!(CardType = Debit)),
            #[cfg(feature = "v2")]
            global_enums::PaymentMethodType::Card => Ok(dirval!(CardType = Card)),
            global_enums::PaymentMethodType::Giropay => Ok(dirval!(BankRedirectType = Giropay)),
            global_enums::PaymentMethodType::Ideal => Ok(dirval!(BankRedirectType = Ideal)),
            global_enums::PaymentMethodType::Sofort => Ok(dirval!(BankRedirectType = Sofort)),
            global_enums::PaymentMethodType::DuitNow => Ok(dirval!(RealTimePaymentType = DuitNow)),
            global_enums::PaymentMethodType::Eps => Ok(dirval!(BankRedirectType = Eps)),
            global_enums::PaymentMethodType::Fps => Ok(dirval!(RealTimePaymentType = Fps)),
            global_enums::PaymentMethodType::Klarna => Ok(dirval!(PayLaterType = Klarna)),
            global_enums::PaymentMethodType::Affirm => Ok(dirval!(PayLaterType = Affirm)),
            global_enums::PaymentMethodType::AfterpayClearpay => {
                Ok(dirval!(PayLaterType = AfterpayClearpay))
            }
            global_enums::PaymentMethodType::AmazonPay => Ok(dirval!(WalletType = AmazonPay)),
            global_enums::PaymentMethodType::Skrill => Ok(dirval!(WalletType = Skrill)),
            global_enums::PaymentMethodType::Paysera => Ok(dirval!(WalletType = Paysera)),
            global_enums::PaymentMethodType::GooglePay => Ok(dirval!(WalletType = GooglePay)),
            global_enums::PaymentMethodType::Bluecode => Ok(dirval!(WalletType = Bluecode)),
            global_enums::PaymentMethodType::ApplePay => Ok(dirval!(WalletType = ApplePay)),
            global_enums::PaymentMethodType::Paypal => Ok(dirval!(WalletType = Paypal)),
            global_enums::PaymentMethodType::RevolutPay => Ok(dirval!(WalletType = RevolutPay)),
            global_enums::PaymentMethodType::CryptoCurrency => {
                Ok(dirval!(CryptoType = CryptoCurrency))
            }
            global_enums::PaymentMethodType::Ach => match self.1 {
                global_enums::PaymentMethod::BankDebit => Ok(dirval!(BankDebitType = Ach)),
                global_enums::PaymentMethod::BankTransfer => Ok(dirval!(BankTransferType = Ach)),
                global_enums::PaymentMethod::PayLater
                | global_enums::PaymentMethod::Card
                | global_enums::PaymentMethod::CardRedirect
                | global_enums::PaymentMethod::Wallet
                | global_enums::PaymentMethod::BankRedirect
                | global_enums::PaymentMethod::Crypto
                | global_enums::PaymentMethod::Reward
                | global_enums::PaymentMethod::RealTimePayment
                | global_enums::PaymentMethod::Upi
                | global_enums::PaymentMethod::Voucher
                | global_enums::PaymentMethod::OpenBanking
                | global_enums::PaymentMethod::MobilePayment
                | global_enums::PaymentMethod::GiftCard => Err(AnalysisErrorType::NotSupported),
            },
            global_enums::PaymentMethodType::Bacs => match self.1 {
                global_enums::PaymentMethod::BankDebit => Ok(dirval!(BankDebitType = Bacs)),
                global_enums::PaymentMethod::BankTransfer => Ok(dirval!(BankTransferType = Bacs)),
                global_enums::PaymentMethod::PayLater
                | global_enums::PaymentMethod::Card
                | global_enums::PaymentMethod::CardRedirect
                | global_enums::PaymentMethod::Wallet
                | global_enums::PaymentMethod::BankRedirect
                | global_enums::PaymentMethod::Crypto
                | global_enums::PaymentMethod::Reward
                | global_enums::PaymentMethod::RealTimePayment
                | global_enums::PaymentMethod::Upi
                | global_enums::PaymentMethod::Voucher
                | global_enums::PaymentMethod::OpenBanking
                | global_enums::PaymentMethod::MobilePayment
                | global_enums::PaymentMethod::GiftCard => Err(AnalysisErrorType::NotSupported),
            },
            global_enums::PaymentMethodType::Becs => Ok(dirval!(BankDebitType = Becs)),
            global_enums::PaymentMethodType::Sepa => Ok(dirval!(BankDebitType = Sepa)),
            global_enums::PaymentMethodType::SepaBankTransfer => {
                Ok(dirval!(BankTransferType = SepaBankTransfer))
            }
            global_enums::PaymentMethodType::AliPay => Ok(dirval!(WalletType = AliPay)),
            global_enums::PaymentMethodType::AliPayHk => Ok(dirval!(WalletType = AliPayHk)),
            global_enums::PaymentMethodType::BancontactCard => {
                Ok(dirval!(BankRedirectType = BancontactCard))
            }
            global_enums::PaymentMethodType::Blik => Ok(dirval!(BankRedirectType = Blik)),
            global_enums::PaymentMethodType::MbWay => Ok(dirval!(WalletType = MbWay)),
            global_enums::PaymentMethodType::MobilePay => Ok(dirval!(WalletType = MobilePay)),
            global_enums::PaymentMethodType::Cashapp => Ok(dirval!(WalletType = Cashapp)),
            global_enums::PaymentMethodType::Multibanco => {
                Ok(dirval!(BankTransferType = Multibanco))
            }
            global_enums::PaymentMethodType::Pix => Ok(dirval!(BankTransferType = Pix)),
            global_enums::PaymentMethodType::Pse => Ok(dirval!(BankTransferType = Pse)),
            global_enums::PaymentMethodType::Interac => Ok(dirval!(BankRedirectType = Interac)),
            global_enums::PaymentMethodType::OnlineBankingCzechRepublic => {
                Ok(dirval!(BankRedirectType = OnlineBankingCzechRepublic))
            }
            global_enums::PaymentMethodType::OnlineBankingFinland => {
                Ok(dirval!(BankRedirectType = OnlineBankingFinland))
            }
            global_enums::PaymentMethodType::OnlineBankingPoland => {
                Ok(dirval!(BankRedirectType = OnlineBankingPoland))
            }
            global_enums::PaymentMethodType::OnlineBankingSlovakia => {
                Ok(dirval!(BankRedirectType = OnlineBankingSlovakia))
            }
            global_enums::PaymentMethodType::Swish => Ok(dirval!(WalletType = Swish)),
            global_enums::PaymentMethodType::Trustly => Ok(dirval!(BankRedirectType = Trustly)),
            global_enums::PaymentMethodType::Bizum => Ok(dirval!(BankRedirectType = Bizum)),

            global_enums::PaymentMethodType::PayBright => Ok(dirval!(PayLaterType = PayBright)),
            global_enums::PaymentMethodType::Flexiti => Ok(dirval!(PayLaterType = Flexiti)),
            global_enums::PaymentMethodType::Walley => Ok(dirval!(PayLaterType = Walley)),
            global_enums::PaymentMethodType::Breadpay => Ok(dirval!(PayLaterType = Breadpay)),
            global_enums::PaymentMethodType::Przelewy24 => {
                Ok(dirval!(BankRedirectType = Przelewy24))
            }
            global_enums::PaymentMethodType::PromptPay => {
                Ok(dirval!(RealTimePaymentType = PromptPay))
            }
            global_enums::PaymentMethodType::WeChatPay => Ok(dirval!(WalletType = WeChatPay)),

            global_enums::PaymentMethodType::ClassicReward => {
                Ok(dirval!(RewardType = ClassicReward))
            }
            global_enums::PaymentMethodType::Evoucher => Ok(dirval!(RewardType = Evoucher)),
            global_enums::PaymentMethodType::UpiCollect => Ok(dirval!(UpiType = UpiCollect)),
            global_enums::PaymentMethodType::UpiIntent => Ok(dirval!(UpiType = UpiIntent)),
            global_enums::PaymentMethodType::SamsungPay => Ok(dirval!(WalletType = SamsungPay)),
            global_enums::PaymentMethodType::GoPay => Ok(dirval!(WalletType = GoPay)),
            global_enums::PaymentMethodType::KakaoPay => Ok(dirval!(WalletType = KakaoPay)),
            global_enums::PaymentMethodType::Twint => Ok(dirval!(WalletType = Twint)),
            global_enums::PaymentMethodType::Gcash => Ok(dirval!(WalletType = Gcash)),
            global_enums::PaymentMethodType::Vipps => Ok(dirval!(WalletType = Vipps)),
            global_enums::PaymentMethodType::VietQr => Ok(dirval!(RealTimePaymentType = VietQr)),
            global_enums::PaymentMethodType::Momo => Ok(dirval!(WalletType = Momo)),
            global_enums::PaymentMethodType::Alma => Ok(dirval!(PayLaterType = Alma)),
            global_enums::PaymentMethodType::Dana => Ok(dirval!(WalletType = Dana)),
            global_enums::PaymentMethodType::OnlineBankingFpx => {
                Ok(dirval!(BankRedirectType = OnlineBankingFpx))
            }
            global_enums::PaymentMethodType::LocalBankRedirect => {
                Ok(dirval!(BankRedirectType = LocalBankRedirect))
            }
            global_enums::PaymentMethodType::OnlineBankingThailand => {
                Ok(dirval!(BankRedirectType = OnlineBankingThailand))
            }
            global_enums::PaymentMethodType::TouchNGo => Ok(dirval!(WalletType = TouchNGo)),
            global_enums::PaymentMethodType::Atome => Ok(dirval!(PayLaterType = Atome)),
            global_enums::PaymentMethodType::Boleto => Ok(dirval!(VoucherType = Boleto)),
            global_enums::PaymentMethodType::Efecty => Ok(dirval!(VoucherType = Efecty)),
            global_enums::PaymentMethodType::PagoEfectivo => {
                Ok(dirval!(VoucherType = PagoEfectivo))
            }
            global_enums::PaymentMethodType::RedCompra => Ok(dirval!(VoucherType = RedCompra)),
            global_enums::PaymentMethodType::RedPagos => Ok(dirval!(VoucherType = RedPagos)),
            global_enums::PaymentMethodType::Alfamart => Ok(dirval!(VoucherType = Alfamart)),
            global_enums::PaymentMethodType::BcaBankTransfer => {
                Ok(dirval!(BankTransferType = BcaBankTransfer))
            }
            global_enums::PaymentMethodType::BniVa => Ok(dirval!(BankTransferType = BniVa)),
            global_enums::PaymentMethodType::BriVa => Ok(dirval!(BankTransferType = BriVa)),
            global_enums::PaymentMethodType::CimbVa => Ok(dirval!(BankTransferType = CimbVa)),
            global_enums::PaymentMethodType::DanamonVa => Ok(dirval!(BankTransferType = DanamonVa)),
            global_enums::PaymentMethodType::Indomaret => Ok(dirval!(VoucherType = Indomaret)),
            global_enums::PaymentMethodType::MandiriVa => Ok(dirval!(BankTransferType = MandiriVa)),
            global_enums::PaymentMethodType::LocalBankTransfer => {
                Ok(dirval!(BankTransferType = LocalBankTransfer))
            }
            global_enums::PaymentMethodType::InstantBankTransfer => {
                Ok(dirval!(BankTransferType = InstantBankTransfer))
            }
            global_enums::PaymentMethodType::InstantBankTransferFinland => {
                Ok(dirval!(BankTransferType = InstantBankTransferFinland))
            }
            global_enums::PaymentMethodType::InstantBankTransferPoland => {
                Ok(dirval!(BankTransferType = InstantBankTransferPoland))
            }
            global_enums::PaymentMethodType::PermataBankTransfer => {
                Ok(dirval!(BankTransferType = PermataBankTransfer))
            }
            global_enums::PaymentMethodType::PaySafeCard => Ok(dirval!(GiftCardType = PaySafeCard)),
            global_enums::PaymentMethodType::SevenEleven => Ok(dirval!(VoucherType = SevenEleven)),
            global_enums::PaymentMethodType::Lawson => Ok(dirval!(VoucherType = Lawson)),
            global_enums::PaymentMethodType::MiniStop => Ok(dirval!(VoucherType = MiniStop)),
            global_enums::PaymentMethodType::FamilyMart => Ok(dirval!(VoucherType = FamilyMart)),
            global_enums::PaymentMethodType::Seicomart => Ok(dirval!(VoucherType = Seicomart)),
            global_enums::PaymentMethodType::PayEasy => Ok(dirval!(VoucherType = PayEasy)),
            global_enums::PaymentMethodType::Givex => Ok(dirval!(GiftCardType = Givex)),
            global_enums::PaymentMethodType::Benefit => Ok(dirval!(CardRedirectType = Benefit)),
            global_enums::PaymentMethodType::Knet => Ok(dirval!(CardRedirectType = Knet)),
            global_enums::PaymentMethodType::OpenBankingUk => {
                Ok(dirval!(BankRedirectType = OpenBankingUk))
            }
            global_enums::PaymentMethodType::MomoAtm => Ok(dirval!(CardRedirectType = MomoAtm)),
            global_enums::PaymentMethodType::Oxxo => Ok(dirval!(VoucherType = Oxxo)),
            global_enums::PaymentMethodType::CardRedirect => {
                Ok(dirval!(CardRedirectType = CardRedirect))
            }
            global_enums::PaymentMethodType::Venmo => Ok(dirval!(WalletType = Venmo)),
            global_enums::PaymentMethodType::Mifinity => Ok(dirval!(WalletType = Mifinity)),
            global_enums::PaymentMethodType::OpenBankingPIS => {
                Ok(dirval!(OpenBankingType = OpenBankingPIS))
            }
            global_enums::PaymentMethodType::Paze => Ok(dirval!(WalletType = Paze)),
            global_enums::PaymentMethodType::DirectCarrierBilling => {
                Ok(dirval!(MobilePaymentType = DirectCarrierBilling))
            }
            global_enums::PaymentMethodType::Eft => Ok(dirval!(BankRedirectType = Eft)),
            global_enums::PaymentMethodType::IndonesianBankTransfer => {
                Ok(dirval!(BankTransferType = IndonesianBankTransfer))
            }
        }
    }
}
