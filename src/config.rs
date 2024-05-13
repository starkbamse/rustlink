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
use std::{path::Path, str::FromStr};
use crate::{
    error::Error, fetcher::fetch_rounds, interface::PriceData
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
pub struct Rustlink {
    pub configuration: Configuration,
    pub tree: Tree,
}



impl Rustlink {
    /// Creates a new Rustlink instance.
    ///
    /// Expected parameters:
    /// - `rpc_url`: The RPC url of your chosen EVM network where Chainlink offers decentralised data feeds.
    /// Don't know where? Check https://data.chain.link/feeds.
    /// - `fetch_interval_seconds`: How often to update data points in the database (to prevent RPC rate limitation)
    /// - `sled_path`: The path for the database
    /// - `contracts`: A tuple list containing a ticker name and its corresponding contract address on the
    /// EVM chain.
    ///
    /// Example:
    ///
    /// ```rust
    /// use rustlink::config::Rustlink;
    /// let mut contracts:Vec<(String,String)>=Vec::new();
    /// contracts.push(("BTC".to_string(),"0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e".to_string()));
    /// let crypto_prices=Rustlink::try_new("https://bsc-dataseed1.binance.org/",10,"./rustlink",contracts);
    /// ```
    pub fn try_new<P : AsRef<Path>>(
        rpc_url: &str,
        fetch_interval_seconds: u64,
        sled_path: P,
        contracts: Vec<(String, String)>,
    ) -> Result<Self,Error> {
        let db = sled::open(sled_path.as_ref())?;
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

        Ok(Rustlink {
            configuration: Configuration {
                fetch_interval_seconds,
                provider,
                contracts,
            },
            tree: db.open_tree("rustlink").unwrap(),
        })
    }

    /// Starts fetching data points in a new asynchronous task
    /// from the chainlink contracts.
    pub fn fetch(&self) {
        tokio::spawn(fetch_rounds(self.clone()));
    }

    /// Helper function to retrieve data from the Sled db.
    fn get_from_tree(db: &Tree, key: &str) -> Result<Vec<u8>, Error> {
        Ok(db.get(key)?.ok_or(Error::NotFound)?.to_vec())
    }

    /// Public getter for reading the latest retrieved cryptocurrency price.
    pub fn get_answer(&self, ticker: &str) -> Result<PriceData, Error> {
        let binary_data = Self::get_from_tree(&self.tree, ticker)?;
        bincode::deserialize::<PriceData>(&binary_data).map_err(|error| {
            log::error!("Deserialization: {}", error);
            Error::Deserialize
        })
    }
}