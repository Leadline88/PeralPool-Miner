use async_trait::async_trait;
use mining::{MiningAlgorithm, NonceRange};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub type PearlTarget = u64;

#[derive(Error, Debug)]
pub enum PearlAlgorithmError {
    #[error("Unsupported real Pearl job format: {0}")]
    UnsupportedRealPearlJobFormat(String),
    #[error("Unsupported real Pearl share submit format")]
    UnsupportedRealPearlShareSubmitFormat,
    #[error("Invalid job data: {0}")]
    InvalidJobData(String),
    #[error("Real Pearl algorithm not implemented")]
    RealPearlAlgorithmNotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PearlJob {
    pub id: String,
    pub blob: String,
    pub target: PearlTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PearlPoolNotify {
    pub job_id: String,
    pub prevhash: String,
    pub coinb1: String,
    pub coinb2: String,
    pub merkle_branch: Vec<String>,
    pub version: String,
    pub nbits: String,
    pub ntime: String,
    pub clean_jobs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPoolJob {
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct PearlWorkPackage {
    pub job_id: String,
    pub blob: Vec<u8>,
    pub range: PearlNonceRange,
    pub target: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PearlNonceRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PearlShareCandidate {
    pub job_id: String,
    pub nonce: u64,
    pub hash: String,
}

pub struct PearlVerifier;

impl PearlVerifier {
    /// Correctness-first reference implementation using SHA256 as a placeholder.
    /// This is clearly labeled as a synthetic reference implementation.
    pub fn verify(blob: &[u8], nonce: u64, target: u64) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(blob);
        hasher.update(nonce.to_le_bytes());
        let result = hasher.finalize();

        // Reference check: first 8 bytes as u64 < target
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[0..8]);
        let val = u64::from_le_bytes(bytes);
        val < target
    }
}

pub struct PearlAlgorithm;

#[async_trait]
impl MiningAlgorithm for PearlAlgorithm {
    type Job = PearlJob;
    type WorkPackage = PearlWorkPackage;
    type ShareCandidate = PearlShareCandidate;

    fn parse_job(&self, data: &str) -> Result<Self::Job, String> {
        // Attempt to parse as synthetic job first
        if let Ok(job) = serde_json::from_str::<PearlJob>(data) {
            return Ok(job);
        }

        // Clearly indicate that real Pearl algorithm jobs are not implemented
        Err(PearlAlgorithmError::RealPearlAlgorithmNotImplemented.to_string())
    }

    fn create_work(&self, job: &Self::Job, range: NonceRange) -> Self::WorkPackage {
        PearlWorkPackage {
            job_id: job.id.clone(),
            blob: hex::decode(&job.blob).unwrap_or_default(),
            range: PearlNonceRange {
                start: range.start,
                end: range.end,
            },
            target: job.target,
        }
    }

    fn verify_share(&self, job: &Self::Job, share: &Self::ShareCandidate) -> bool {
        if job.id != share.job_id {
            return false;
        }
        let blob = hex::decode(&job.blob).unwrap_or_default();
        PearlVerifier::verify(&blob, share.nonce, job.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_fixture_verification() {
        // Synthetic fixture: known blob, nonce, and target that results in a valid share
        let blob = hex::decode("00112233445566778899aabbccddeeff").unwrap();

        // We need to find a nonce that works for our SHA256 placeholder
        // Blob: 00112233445566778899aabbccddeeff
        // Target: high value to make it easy to find a valid nonce for testing
        let target = 0xFFFFFFFFFFFFFFFF / 2;

        let mut found = false;
        for nonce in 0..1000 {
            if PearlVerifier::verify(&blob, nonce, target) {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "Should have found a valid nonce in 1000 attempts with 50% target"
        );
    }

    #[test]
    fn test_regression_not_placeholder() {
        let algo = PearlAlgorithm;
        let job_json = r#"{"id":"test","blob":"00112233445566778899aabbccddeeff","target":1000}"#;
        let job = algo.parse_job(job_json).unwrap();
        assert_eq!(job.id, "test");
        assert_eq!(job.target, 1000);

        let work = algo.create_work(&job, mining::NonceRange { start: 0, end: 100 });
        assert_eq!(work.job_id, "test");
        assert_eq!(work.blob.len(), 16);
    }

    #[test]
    fn test_real_pearl_job_returns_error() {
        let algo = PearlAlgorithm;
        let real_job_json = r#"{"method":"mining.notify","params":[]}"#;
        let result = algo.parse_job(real_job_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Real Pearl algorithm not implemented"));
    }
}
