mod fetcher;
mod interface;
use alloy::{
    providers::{ProviderBuilder, RootProvider},
    transports::http::Http,
};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use reqwest::{Client, Url};
use sled::Tree;
use std::str::FromStr;
use thiserror::Error;

use self::{
    fetcher::fetch_prices,
    interface::{PriceData, Serializable},
};

#[derive(Clone)]
pub struct Configuration {
    pub fetch_interval_seconds: u64,
    pub contracts: Vec<(String, String)>,
    pub provider: RootProvider<Http<Client>>,
}

/// ## Cryptoprices
///
/// Cryptoprices is a lightweight Rust library that provides your Rust applications with a direct
/// link to the latest cryptocurrency prices. All data is retrieved from Chainlink decentralized
/// price feeds. Just copy the contract addresses for the symbol that you would like to track from:
/// https://data.chain.link/feeds.
///
///
/// Note: Rustlink is designed to be ran on the main thread, in an asynchronous tokio environment
#[derive(Clone)]
pub struct CryptoPrices {
    configuration: Configuration,
    tree: Tree,
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Could not find symbol")]
    NotFound,
    #[error("Could not deserialize binary data")]
    Deserialize,
    #[error("Database internal error: {0}")]
    SledError(#[from] sled::Error),
}

impl CryptoPrices {
    /// Creates a new Rustlink instance.
    ///
    /// Expected parameters:
    /// - `rpc_url`: The RPC url of your chosen EVM network where Chainlink offers decentralised data feeds.
    /// Don't know where? Check https://data.chain.link/feeds.
    /// - `fetch_interval_seconds`: How often to update prices in the database (to prevent RPC rate limitation)
    /// - `sled_path`: The path for the database
    /// - `contracts`: A tuple list containing a ticker name and its corresponding contract address on the
    /// EVM chain.
    ///
    /// Example:
    ///
    /// ```rust
    /// use cryptoprices::CryptoPrices;
    /// let mut contracts:Vec<(String,String)>=Vec::new();
    /// contracts.push(("BTC".to_string(),"0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e".to_string()));
    /// let crypto_prices=CryptoPrices::new("https://bsc-dataseed1.binance.org/",10,"./cryptoprices",contracts);
    /// ```
    pub fn new(
        rpc_url: &str,
        fetch_interval_seconds: u64,
        sled_path: &str,
        contracts: Vec<(String, String)>,
    ) -> CryptoPrices {
        let db = sled::open(sled_path).unwrap();
        let provider = ProviderBuilder::new().on_http(Url::from_str(rpc_url).unwrap());

        // Setup logging
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build("./rustlink.log")
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(LevelFilter::Info))
            .unwrap();

        // Try to initialize and catch error silently if already initialized
        // during tests this make this function throw error
        if log4rs::init_config(config).is_err() {
            println!("Logger already initialized.");
        }

        CryptoPrices {
            configuration: Configuration {
                fetch_interval_seconds,
                provider,
                contracts,
            },
            tree: db.open_tree("rustlink").unwrap(),
        }
    }

    /// Starts fetching prices in a new asynchronous task
    /// from the chainlink contracts.
    pub fn fetch(&self) {
        tokio::spawn(fetch_prices(self.clone()));
    }

    /// Helper function to retrieve data from the Sled db.
    fn get_from_tree(db: &Tree, key: &str) -> Result<Vec<u8>, DatabaseError> {
        Ok(db.get(key)?.ok_or(DatabaseError::NotFound)?.to_vec())
    }

    /// Public getter for reading the latest retrieved cryptocurrency price.
    pub fn get_price(&self, ticker: &str) -> Result<PriceData, DatabaseError> {
        let binary_data = Self::get_from_tree(&self.tree, ticker)?;
        PriceData::from_bin(binary_data).map_err(|error| {
            log::error!("Deserialization: {}", error);
            DatabaseError::Deserialize
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::CryptoPrices;
    use std::{fs, path::Path};
    fn remove_test_db(db_path: &str) {
        if Path::new(db_path).exists() {
            fs::remove_dir_all(db_path).expect("Failed to remove test database");
        }
    }
    #[tokio::test]
    async fn ensure_price_is_received() {
        let mut contracts: Vec<(String, String)> = Vec::new();
        contracts.push((
            "BTC".to_string(),
            "0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e".to_string(),
        ));
        let crypto_prices = CryptoPrices::new(
            "https://bsc-dataseed1.binance.org/",
            1,
            "./test-crypto-prices",
            contracts,
        );

        crypto_prices.fetch();

        // Within 10 seconds we can confidently check for price
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let btc_price = crypto_prices.get_price("BTC").unwrap();
        println!("ETH Price: {}", btc_price.price);
        println!("Updated at: {}", btc_price.updated_at);
        remove_test_db("./test-crypto-prices");
        assert!(btc_price.price.ge(&0f64));
    }
}
