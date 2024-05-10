use std::str::FromStr;

use sled::Tree;

use crate::chains::Chain;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use log::LevelFilter;
use alloy::{
    primitives::{Address, U256},
    providers::{ProviderBuilder, RootProvider},
    signers::wallet::LocalWallet,
    transports::http::Http,
};
use reqwest::{Client, Url};
pub struct RustlinkConfiguration {
    pub chain:Chain,
    pub fetch_interval_seconds:u64,
    pub provider:RootProvider<Http<Client>>
}

pub struct Rustlink {
    pub configuration:RustlinkConfiguration,
    pub tree:Tree
}

impl Rustlink {
    pub fn new(chain:Chain,fetch_interval_seconds:u64,sled_path:&str)->Rustlink{
        let db=sled::open(sled_path).unwrap();
        let provider = ProviderBuilder::new().on_http(Url::from_str(chain.rpc_url()).unwrap());

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

        Rustlink {
            configuration:RustlinkConfiguration{
                chain,
                fetch_interval_seconds,
                provider
            },
            tree:db.open_tree("rustlink").unwrap()
        }
    
    }

    pub fn fetch() {
        //tokio::spawn();

    }

}
    
