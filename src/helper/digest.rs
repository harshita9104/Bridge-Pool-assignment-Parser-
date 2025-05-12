use sha2::{Sha256, Digest as Sha2Digest};

pub trait Digest {
    fn hash_bytes(&self, input: &[u8]) -> String;
    fn hash_entry(&self, line: &[u8], file_sha: &str) -> String;
}

pub struct Sha256Digest;

impl Digest for Sha256Digest {
    fn hash_bytes(&self, input: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input);
        hex::encode(hasher.finalize())
    }

    fn hash_entry(&self, line: &[u8], file_sha: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(line);
        hasher.update(file_sha.as_bytes());
        hex::encode(hasher.finalize())
    }
}
