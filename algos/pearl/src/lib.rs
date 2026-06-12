use async_trait::async_trait;
use mining::{MiningAlgorithm, NonceRange};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type PearlTarget = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PearlJob {
    pub id: String,
    pub blob: String,
    pub target: PearlTarget,
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
        serde_json::from_str(data).map_err(|e| e.to_string())
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
}
