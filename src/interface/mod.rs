use alloy::{contract::Error, primitives::Uint, providers::RootProvider, sol, transports::http::Http};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use self::IAggregatorV3Interface::IAggregatorV3InterfaceInstance;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IAggregatorV3Interface,
    "src/abi/IAggregatorV3Interface.json"
);

#[derive(Clone)]
pub struct ChainlinkContract<'a> {
    pub contract: IAggregatorV3InterfaceInstance<Http<Client>, &'a RootProvider<Http<Client>>>,
    pub decimals: u8,
}

/// The latest price received for this symbol.
/// This data is directly retrieved from the underlying contract.
#[derive(Deserialize, Serialize,Debug)]
pub struct PriceData {
    /// Id of the submission by the aggregator
    pub round_id: u128,
    /// Answered in round
    pub answered_in_round: u128,
    /// Timestamp for when the aggregator started collecting data
    pub started_at: Uint<256,4>,
    /// Timestamp for when the aggregator posted the price update
    pub updated_at: Uint<256,4>,
    /// Price of the asset         
    pub price: f64,
}



impl<'a> ChainlinkContract<'a> {
    /// Creates a new instance of a chainlink price aggregator. This is just a wrapper
    /// function to simplify the interactions with the contract.
    pub async fn new(
        provider: &'a RootProvider<Http<Client>>,
        contract_address: &str,
    ) -> Result<ChainlinkContract <'a>, Error> {
        let contract = IAggregatorV3Interface::new(contract_address.parse().unwrap(), provider);

        let IAggregatorV3Interface::decimalsReturn { _0: decimals } =
            contract.decimals().call().await?;
        Ok(ChainlinkContract { contract, decimals })
    }

    /// Retrieves the latest price of this underlying asset
    /// from the chainlink decentralized data feed
    pub async fn latest_round_data(&self) -> Result<PriceData, Error> {
        let IAggregatorV3Interface::latestRoundDataReturn {
            roundId,
            answer,
            startedAt,
            updatedAt,
            answeredInRound,
        } = self.contract.latestRoundData().call().await?;

        // Convert the price on contract to a string.
        let float_price: f64 = answer.to_string().parse().unwrap();

        // Convert the contract answer into a human-readable price
        let human_price = float_price / (10f64.powi(self.decimals.into()));

        Ok(PriceData {
            round_id: roundId,
            answered_in_round: answeredInRound,
            started_at: startedAt,
            updated_at: updatedAt,
            price: human_price,
        })
    }
}

#[cfg(test)]
mod tests {

    use alloy::providers::ProviderBuilder;
    use reqwest::Url;
    use std::str::FromStr;

    use crate::interface::ChainlinkContract;

    #[tokio::test]
    async fn valid_answer() {
        let provider = ProviderBuilder::new()
            .on_http(Url::from_str("https://bsc-dataseed1.binance.org/").unwrap());

        let chainlink_contract =
            ChainlinkContract::new(&provider, "0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e")
                .await
                .unwrap();
        let price_data = chainlink_contract.latest_round_data().await.unwrap();
        println!("Received data: {:#?}", price_data);
        assert!(price_data.price.ge(&0f64));
    }
}
