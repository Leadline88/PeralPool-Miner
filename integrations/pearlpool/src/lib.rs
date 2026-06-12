pub const DEFAULT_POOL_URL: &str = "stratum+tcp://pearlpool.cloud:5566";
pub const DEFAULT_ALGO: &str = "pearl";

pub fn get_default_profile() -> String {
    format!("Pool URL: {}, Algo: {}", DEFAULT_POOL_URL, DEFAULT_ALGO)
}
