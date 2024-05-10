
mod contracts;
use std::collections::HashMap;

use self::contracts::{arbitrum_contracts, ethereum_contracts};


pub enum Chain {
    Ethereum {
        rpc_url: String,
        contracts: HashMap<String, String>,
    },
    ArbitrumOne {
        rpc_url: String,
        contracts: HashMap<String, String>,
    }
}

/// Chain presets for rustlink. 
/// Available networks:
/// - Ethereum (chain id: 1)
/// - ArbitrumOne (chain id: 42161)
/// 
/// Usage:
/// 
/// ```rust
/// Chain::new(1,"https://1rpc.io")
/// ```
impl Chain{
    /// Create a new chain instance specifying a custom rpc url
    pub fn new(chain_id: u32,rpc_url: String,) -> Self {
        let contracts = match chain_id {
            1 => ethereum_contracts(),
            42161 => arbitrum_contracts(),
            _ => HashMap::new(),
        };

        match chain_id {
            1 => Chain::Ethereum { rpc_url, contracts },
            42161 => Chain::ArbitrumOne { rpc_url, contracts },
            _ => unreachable!(), // Assuming these are the only options
        }
    }

    /// Return the RPC url of this chain.
    pub fn rpc_url(&self) -> &String {
        match self {
            Chain::Ethereum { rpc_url: url, .. } => url,
            Chain::ArbitrumOne { rpc_url: url, .. } => url,
        }
    }
    
    /// Returns a reference to the contracts map for the given chain instance.
    /// If no contracts exist for the chain, returns an empty HashMap.
    pub fn contracts(&self) -> &HashMap<String,String> {
        match self {
            Chain::Ethereum { contracts, .. } => contracts,
            Chain::ArbitrumOne { contracts, .. } => contracts,
        }
    }

}
#[cfg(test)]
mod tests {
    use crate::chains::Chain;

    #[test]
    fn valid_networks(){
        assert!(Chain::new(1,"https://1rpc.io/eth".to_string()).rpc_url().contains("https://"));
        assert!(Chain::new(42161,"https://1rpc.io/arb".to_string()).rpc_url().contains("https://"));
    }

}

