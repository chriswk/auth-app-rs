use std::borrow::Borrow;
use argon2::{self, Config, ThreadMode, Variant, Version};


pub fn hash_password(salt: String, password: String) -> String {
    let config = Config {
        ad: &[],
        hash_length: 10,
        lanes: 4,
        mem_cost: 65536,
        secret: &[],
        thread_mode: ThreadMode::Parallel,
        time_cost: 10,
        variant: Variant::Argon2i,
        version: Version::Version13
    };
    argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), config.borrow()).unwrap()
}

pub fn verify_password(hash: String, password: String) -> bool {
    argon2::verify_encoded(hash.as_str(), password.as_bytes()).unwrap()
}
