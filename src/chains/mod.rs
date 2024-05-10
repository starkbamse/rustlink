
mod contracts;
use std::collections::HashMap;

use self::contracts::{arbitrum_contracts, ethereum_contracts};


enum Chain {
    Ethereum {
        rpc_url: Option<&'static str>,
        contracts: HashMap<&'static str, &'static str>,
    },
    ArbitrumOne {
        rpc_url: Option<&'static str>,
        contracts: HashMap<&'static str, &'static str>,
    }
}

/// Chain presets for rustlink. 
/// Available networks:
/// - Ethereum (chain id: 1)
/// - ArbitrumOne (chain id: 42161)
/// 
/// Usage:
/// 
/// **Using fallback RPC**
/// ```rust
/// Chain::new(1,None)
/// ```
/// 
/// **Using custom RPC url**
/// ```rust
/// Chain::new(1,"https://1rpc.io")
/// ```
impl Chain{
    /// Create a new chain instance optionally specifying a custom rpc url
    pub fn new(chain_id: u32,rpc_url: Option<&'static str>,) -> Self {
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
    pub fn rpc_url(&self) -> &str {
        match self {
            Chain::Ethereum { rpc_url: Some(url), .. } => url,
            Chain::Ethereum { rpc_url: None, .. } => "https://1rpc.io/eth",
            
            Chain::ArbitrumOne { rpc_url: Some(url), .. } => url,
            Chain::ArbitrumOne { rpc_url: None, .. } => "https://1rpc.io/arb",
        }
    }
    
    /// Returns a reference to the contracts map for the given chain instance.
    /// If no contracts exist for the chain, returns an empty HashMap.
    pub fn contracts(&self) -> &HashMap<&'static str, &'static str> {
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
        assert!(Chain::new(1,None).rpc_url().contains("https://"));
        assert!(Chain::new(42161,None).rpc_url().contains("https://"));

        
    }

}

