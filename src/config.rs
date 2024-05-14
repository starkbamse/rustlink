use crate::{error::Error, fetcher::fetch_rounds, interface::Round};
use alloy::{
    providers::{ProviderBuilder, RootProvider},
    transports::http::Http,
};
use async_std::channel::Sender;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use reqwest::{Client, Url};
use std::str::FromStr;

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
    pub reflector: Reflector,
}

/// Rustlink allows you as a developer to retrieve the sought Chainlink data in
/// different ways.
///
/// In its current state you can pass a Sender from an unbound async-std channel
/// which you can create by doing:
/// ```rust
/// use async_std::channel::unbounded;
/// use rustlink::config::Reflector;
///
/// let (sender, receiver) = unbounded();
///
/// let reflector=Reflector::Sender(sender);
/// ```
///
/// You may clone the receiver as many times as you want but do not use the sender
/// for anything other than passing it to the try_new() method.
#[derive(Clone)]
pub enum Reflector {
    /// A sender from async-std
    Sender(Sender<Round>),
}

impl Rustlink {
    /// Creates a new Rustlink instance.
    ///
    /// Expected parameters:
    /// - `rpc_url`: The RPC url of your chosen EVM network where Chainlink offers decentralised data feeds.
    /// Don't know where? Check https://data.chain.link/feeds.
    /// - `fetch_interval_seconds`: How often to update data points in the database (to prevent RPC rate limitation)
    /// - `reflector`: How you choose to receive the answer from your provided contracts.
    /// - `contracts`: A tuple list containing a ticker name and its corresponding contract address on the
    /// EVM chain.
    ///
    /// Example:
    ///
    /// ```rust
    /// use rustlink::config::Rustlink;
    /// use async_std::channel::unbounded;
    /// use rustlink::config::Reflector;
    /// let mut contracts:Vec<(String,String)>=Vec::new();
    /// contracts.push(("BTC".to_string(),"0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e".to_string()));
    ///
    /// let (sender, receiver) = unbounded();
    ///
    /// let reflector=Reflector::Sender(sender);///
    /// let crypto_prices=Rustlink::try_new("https://bsc-dataseed1.binance.org/",10,reflector,contracts);
    /// ```
    pub fn try_new(
        rpc_url: &str,
        fetch_interval_seconds: u64,
        reflector: Reflector,
        contracts: Vec<(String, String)>,
    ) -> Result<Self, Error> {
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
            reflector,
        })
    }

    /// Starts fetching data points in a new asynchronous task
    /// from the chainlink contracts.
    pub fn fetch(&self) {
        tokio::spawn(fetch_rounds(self.clone()));
    }
}
