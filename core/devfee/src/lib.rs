pub use mining::DevFeeState;

pub const DEFAULT_DEV_FEE: f64 = 1.0;
pub const DEFAULT_DEV_WALLET: &str = "1DevFeeAddressExample";

pub struct DevFee;

impl DevFee {
    pub fn percentage() -> f64 {
        DEFAULT_DEV_FEE
    }

    pub fn wallet() -> &'static str {
        DEFAULT_DEV_WALLET
    }
}
