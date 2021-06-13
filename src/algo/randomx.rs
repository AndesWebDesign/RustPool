use hex::encode;
use randomx_rs::{RandomXCache, RandomXFlag, RandomXVM};

use crate::structs::Config;

pub fn get_rx_hash(config: &Config, input: &[u8], seed_hash: &[u8]) -> String {
    let mut flags = RandomXFlag::get_recommended_flags();
    if config.rx_use_full_memory {
        flags = flags | RandomXFlag::FLAG_FULL_MEM;
    }
    if config.rx_use_large_pages {
        flags = flags | RandomXFlag::FLAG_LARGE_PAGES;
    }
    if config.rx_set_secure_flag {
        flags = flags | RandomXFlag::FLAG_SECURE;
    }
    let cache = RandomXCache::new(flags, seed_hash).unwrap();
    let vm = RandomXVM::new(flags, Some(&cache), None).unwrap();
    let hash_vec = vm.calculate_hash(input).unwrap();
    let hash_hex = encode(hash_vec);
    return hash_hex;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
