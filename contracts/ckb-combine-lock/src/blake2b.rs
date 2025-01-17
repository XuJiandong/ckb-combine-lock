pub use blake2b_rs::{Blake2b, Blake2bBuilder};

pub const CKB_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32).personal(CKB_PERSONALIZATION).build()
}

pub fn hash(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = new_blake2b();
    hasher.update(bytes);

    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    hash
}
