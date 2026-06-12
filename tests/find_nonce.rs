use sha2::{Digest, Sha256};

#[test]
fn find_valid_nonce() {
    let blob = hex::decode("00112233445566778899aabbccddeeff").unwrap();
    let target = 9223372036854775807;
    for nonce in 0..1000 {
        let mut hasher = Sha256::new();
        hasher.update(&blob);
        hasher.update(nonce.to_le_bytes());
        let result = hasher.finalize();
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[0..8]);
        let val = u64::from_le_bytes(bytes);
        if val < target {
            println!("Found nonce: {}", nonce);
            return;
        }
    }
}
