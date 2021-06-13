use std::ops::Div;

use hex::{decode, encode};
use log::warn;
use monero::Block;
use monero::consensus::deserialize;
use monero::cryptonote::hash::Hashable;
use num_bigint::{BigInt, Sign};
use sha3::{Digest, Keccak256};

use crate::structs::Config;

pub fn keccak(input: Vec<u8>, size: usize) -> Vec<u8> {
    assert_eq!(input.len(), size);
    let mut hasher = Keccak256::new();
    hasher.update(input);
    let result = hasher.finalize();
    return result.to_vec();
}

fn get_tree_hash(transaction_hashes: Vec<Vec<u8>>) -> Vec<u8> {
    // a copy of tree_hash function from monero at
    // https://github.com/monero-project/monero/blob/master/src/crypto/tree-hash.c
    let count: usize = transaction_hashes.len();
    let hash_size: usize = transaction_hashes[0].len();
    if count == 1 {
        return transaction_hashes[0].clone();
    } else if count == 2 {
        return keccak(concat_hashes(&transaction_hashes.clone(), 0), 2 * hash_size);
    } else {
        let mut buffer = vec![vec![0 as u8; hash_size]; count];
        let mut cut = 1 << (f64::floor(f64::log2(count as f64)) as u8);
        for i in 0..(2 * cut - (count as i32)) {
            buffer[i as usize] = transaction_hashes[i as usize].clone()
        }
        let i_range = ((2 * cut - (count as i32))..((count as i32) + 1)).step_by(2);
        let j_range = (2 * cut - (count as i32))..cut;
        for (i, j) in i_range.zip(j_range) {
            buffer[j as usize] = keccak(concat_hashes(&transaction_hashes.clone(), i as usize), 2 * hash_size);
        }
        while cut > 2 {
            cut >>= 1;
            let i_range = (0..(2 * cut)).step_by(2);
            let j_range = 0..cut;
            for (i, j) in i_range.zip(j_range) {
                buffer[j as usize] = keccak(concat_hashes(&buffer, i as usize), 2 * hash_size);
            }
        }
        return keccak(concat_hashes(&buffer, 0), 2 * hash_size);
    }
}

fn concat_hashes(transaction_hashes: &Vec<Vec<u8>>, index: usize) -> Vec<u8> {
    let mut input = transaction_hashes[index].clone();
    input.extend(transaction_hashes[index + 1].clone());
    return input;
}

pub fn get_hashing_blob_from_template(blob: &Vec<u8>) -> String {
    match deserialize::<Block>(&blob[..]) {
        Ok(block) => {
            let mut hashes: Vec<Vec<u8>> = Vec::new();
            hashes.push(block.miner_tx.hash().0.to_vec());
            for tx_hash in block.tx_hashes {
                hashes.push(tx_hash.0.to_vec());
            }
            let mut tx_count = format!("{:x}", hashes.len());
            if tx_count.len() % 2 != 0 {
                tx_count = "0".to_owned() + &*tx_count;
            }
            let miner_tx_hash = &get_tree_hash(hashes);
            let mut vec = blob[0..39].to_vec();
            vec.extend(vec![0 as u8; 4]);
            vec.extend(miner_tx_hash);
            let mut hash = encode(vec);
            hash.push_str(&*tx_count);
            return hash;
        }
        Err(_) => {
            return String::new();
        }
    }
}

pub fn format_block_template(config: &Config,
                             blob: &str,
                             pool_nonce: &str,
                             offset: &i32) -> Option<Vec<u8>> {
    let template_vec = &blob.as_bytes().to_vec();
    let mut vec = template_vec[..=(*offset as usize)].to_vec();
    let tp_range = (2 * (config.pool_reserve_size_bytes as usize) + (*offset as usize) + 1)..;
    let third_part = template_vec[tp_range].to_vec();
    vec.extend(decode(pool_nonce).unwrap());
    vec.extend(third_part);
    return Some(vec.clone());
}

pub fn add_miner_nonce(blob: &Vec<u8>, miner_nonce: &str) -> Option<Vec<u8>> {
    let mut vec = blob[..39].to_vec().clone();
    vec.extend(decode(miner_nonce).unwrap().to_vec().clone());
    vec.extend(blob[43..].to_vec().clone());
    return Some(vec.clone());
}

pub fn calculate_difficulty(hash: &String) -> BigInt {
    let actual_vec: Vec<u8> = decode(hash).unwrap();
    let actual_rev: Vec<u8> = actual_vec.into_iter().map(|x| x.to_be()).rev().collect();
    let hash_int: BigInt = BigInt::from_bytes_be(Sign::Plus, actual_rev.as_slice());
    let base_diff_vec: Vec<u8> = vec![u8::MAX; 32];
    let base_diff: BigInt = BigInt::from_bytes_le(Sign::Plus, base_diff_vec.as_slice());
    let hash_diff = base_diff.div(hash_int);
    return hash_diff;
}

pub fn template_valid(config: &Config, blob: &Vec<u8>) -> bool {
    match deserialize::<Block>(&blob[..]) {
        Ok(block) => {
            let miner_tx_outputs = block.miner_tx.prefix.outputs;
            if miner_tx_outputs.len() != 1 {
                warn!("no miner transaction outputs");
                return false;
            }
            let miner_tx_keys = miner_tx_outputs[0].target.get_pubkeys().unwrap_or(Vec::new());
            if miner_tx_keys.len() != 1 {
                warn!("no miner transaction keys");
                return false;
            }
            let miner_tx_output_key = miner_tx_keys[0].to_string();

            if miner_tx_output_key.ne(config.wallet.as_str()) {
                warn!("miner transaction key not pool wallet");
                return false;
            }
            return true;
        }
        Err(_) => {
            return false;
        }
    }
}
