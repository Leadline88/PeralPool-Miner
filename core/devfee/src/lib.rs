use miner::DevFeePolicy;

pub const DEFAULT_DEV_FEE: f64 = 1.0;
pub const DEFAULT_DEV_WALLET: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Placeholder dev wallet

#[derive(Debug)]
pub struct DevFee {
    percentage: f64,
    wallet: String,
}

impl Default for DevFee {
    fn default() -> Self {
        Self {
            percentage: DEFAULT_DEV_FEE,
            wallet: DEFAULT_DEV_WALLET.to_string(),
        }
    }
}

impl DevFee {
    pub fn new(percentage: f64, wallet: String) -> Self {
        Self { percentage, wallet }
    }

    pub fn percentage(&self) -> f64 {
        self.percentage
    }

    pub fn wallet(&self) -> &str {
        &self.wallet
    }
}

impl DevFeePolicy for DevFee {
    fn fee_percentage(&self) -> f64 {
        self.percentage
    }

    fn dev_wallet(&self) -> &str {
        &self.wallet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_fee_is_one_percent() {
        let dev_fee = DevFee::default();
        assert_eq!(dev_fee.percentage(), 1.0);
    }

    #[test]
    fn test_custom_fee() {
        let dev_fee = DevFee::new(2.0, "custom_wallet".to_string());
        assert_eq!(dev_fee.percentage(), 2.0);
        assert_eq!(dev_fee.wallet(), "custom_wallet");
    }

    #[test]
    fn test_dev_fee_policy_trait() {
        let dev_fee = DevFee::default();
        let policy: &dyn DevFeePolicy = &dev_fee;
        assert_eq!(policy.fee_percentage(), 1.0);
        assert_eq!(policy.dev_wallet(), DEFAULT_DEV_WALLET);
    }
}
