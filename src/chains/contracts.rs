// contracts.rs
use std::collections::HashMap;

pub fn ethereum_contracts() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("BTC", "0xabcd1234efgh5678ijkl9012mnop3456qrst7890");
    map.insert("ETH", "0x1234abcd5678efgh9012ijkl3456mnop7890qrst");
    map
}

pub fn arbitrum_contracts() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("LINK", "0x1234abcd5678efgh9012ijkl3456mnop7890qrst");
    map.insert("DAI", "0xabcd1234efgh5678ijkl9012mnop3456qrst7890");
    map
}
