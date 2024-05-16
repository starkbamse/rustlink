use crate::{error::Error, fetcher::fetch_rounds, interface::Round};
use alloy::{
    providers::{ProviderBuilder, RootProvider},
    transports::http::Http,
};
use async_std::channel::{unbounded, Receiver, RecvError, Sender};
use js_sys::Function;
use reqwest::{Client, Url};
use serde_wasm_bindgen::{from_value, to_value};
use workflow_rs::core::cfg_if;
use std::str::FromStr;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use wasm_bindgen_futures::spawn_local;
#[derive(Clone)]
pub struct Configuration {
    pub fetch_interval_seconds: u64,
    pub contracts: Vec<(String, String)>,
    pub provider: RootProvider<Http<Client>>,
}

/// ## Rustlink
///
/// Rustlink is a lightweight Rust library that provides your Rust applications with a direct
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
    pub termination_send: Sender<()>,
    pub termination_recv: Receiver<()>,
    pub shutdown_send: Sender<()>,
    pub shutdown_recv: Receiver<()>,
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
        let (termination_send, termination_recv) = unbounded::<()>();
        let (shutdown_send, shutdown_recv) = unbounded::<()>();
        Ok(Rustlink {
            configuration: Configuration {
                fetch_interval_seconds,
                provider,
                contracts,
            },
            reflector,
            termination_send,
            termination_recv,
            shutdown_send,
            shutdown_recv,
        })
    }

    pub fn start(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        tokio::task::spawn(fetch_rounds(self.clone()));

        #[cfg(target_arch = "wasm32")]
        async_std::task::block_on(fetch_rounds(self.clone()));


    }

    pub async fn stop(&self) -> Result<(), RecvError> {
        self.termination_send.send(()).await.unwrap();
        self.shutdown_recv.recv().await
    }
}
#[wasm_bindgen]
pub struct RustlinkJS {
    rustlink: Rustlink,
    callback: Function,
    receiver: Receiver<Round>,
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        #[wasm_bindgen(typescript_custom_section)]
        const TS_CONTRACTS: &'static str = r#"
        /** 
         * A contract tuple containing an identifier and a contract address. 
         * 
         * **Order matters.**
         * Example
         * ```typescript
         * let contracts=[["Ethereum","0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e"]]
         * ```
        */
        export type Contract = [string,string] 
        "#;

    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Function, typescript_type = "Contract[]")]
    pub type Contracts;
}

#[wasm_bindgen]
impl RustlinkJS {
    /// Creates a new RustlinkJS instance
    #[wasm_bindgen(constructor)]
    pub fn new(
        rpc_url: &str,
        fetch_interval_seconds: u64,
        contracts: Contracts,
        callback: Function,
    ) -> Self {

        
        // Cast `JsValue` to `Function`

        let contracts: Vec<(String, String)> = from_value(contracts.into()).unwrap();

        let (sender, receiver) = async_std::channel::unbounded();
        let reflector = Reflector::Sender(sender);
        let rustlink = Rustlink::try_new(rpc_url, fetch_interval_seconds, reflector, contracts)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .unwrap();

        RustlinkJS {
            rustlink,
            callback,
            receiver,
        }
    }
    #[wasm_bindgen]
    pub fn start(&self) {
        self.rustlink.start();
        let receiver = self.receiver.clone();
        let callback = self.callback.clone();
        spawn_local(async move {
            while let Ok(round) = receiver.recv().await {
                // Prepare arguments to pass to JS function
                let this = JsValue::NULL; // 'this' context for function, null in this case
                let arg_js = to_value(&round).unwrap();

                // Call the function
                let _ = callback.call1(&this, &arg_js);
            }
        });
    }

    #[wasm_bindgen]
    pub async fn stop(&self) -> Result<(), JsValue> {
        self.rustlink
            .stop()
            .await
            .map_err(|e| JsValue::from_str(&format!("Shutdown error: {}", e)))
    }
}
