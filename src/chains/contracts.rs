// contracts.rs
use std::collections::HashMap;

pub fn ethereum_contracts() -> HashMap<String,String> {
    let mut map = HashMap::new();
    map.insert("BTC".to_string(), "0xabcd1234efgh5678ijkl9012mnop3456qrst7890".to_string());
    map.insert("ETH".to_string(), "0x1234abcd5678efgh9012ijkl3456mnop7890qrst".to_string());
    map
}

pub fn arbitrum_contracts() -> HashMap<String,String> {
    let mut map = HashMap::new();
    map.insert("BTC".to_string(), "0xabcd1234efgh5678ijkl9012mnop3456qrst7890".to_string());
    map.insert("ETH".to_string(), "0x1234abcd5678efgh9012ijkl3456mnop7890qrst".to_string());
    map
}
