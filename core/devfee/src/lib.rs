use serde::Serialize;

pub const DEFAULT_DEV_FEE: f64 = 1.0;
pub const DEFAULT_DEV_WALLET: &str = "1DevFeeAddressExample";

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum DevFeeState {
    Disabled,
    ScheduledInactive,
    ActiveUserMining,
    ActiveDeveloperMining,
}

impl std::fmt::Display for DevFeeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DevFeeState::Disabled => write!(f, "Disabled"),
            DevFeeState::ScheduledInactive => write!(f, "ScheduledInactive"),
            DevFeeState::ActiveUserMining => write!(f, "ActiveUserMining"),
            DevFeeState::ActiveDeveloperMining => write!(f, "ActiveDeveloperMining"),
        }
    }
}

pub struct DevFee;

impl DevFee {
    pub fn percentage() -> f64 {
        DEFAULT_DEV_FEE
    }

    pub fn wallet() -> &'static str {
        DEFAULT_DEV_WALLET
    }
}
