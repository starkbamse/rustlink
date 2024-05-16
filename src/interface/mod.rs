use std::sync::Arc;
use serde::{Deserialize, Serialize};
use ethers::{abi::{Abi, AbiError}, contract::Contract, providers::{Http, Provider}, types::{Address, U256}};


#[derive(Clone)]
pub struct ChainlinkContract<'a> {
    pub contract: Contract<&'a Provider<Http>>,
    pub identifier: &'a str,
    pub decimals: u8,
}


/// The latest price received for this symbol.
/// This data is directly retrieved from the underlying contract.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Round {
    // Identifier of the underlying asset
    pub identifier: String,
    /// Id of the submission by the aggregator
    pub round_id: u128,
    /// Answered in round
    pub answered_in_round: u128,
    /// Timestamp for when the aggregator started collecting data
    pub started_at: U256,
    /// Timestamp for when the aggregator posted the price update
    pub updated_at: U256,
    /// Answer of this round         
    pub answer: f64,
}

impl<'a> ChainlinkContract<'a> {
    /// Creates a new instance of a chainlink price aggregator. This is just a wrapper
    /// function to simplify the interactions with the contract.
    pub async fn new(
        provider: &'a Provider<Http>,
        identifier: &'a str,
        contract_address: Address,
    ) -> Result<ChainlinkContract<'a>, AbiError> {
        let abi:Abi=serde_json::from_str(include_str!("IAggregatorV3Interface.json")).unwrap();
        let contract = Contract::new(contract_address, abi, Arc::new(provider));

        let decimals=contract.method::<_,U256>("decimals", ()).unwrap()
        .call().await.unwrap().as_u64() as u8;

        Ok(ChainlinkContract {
            contract,
            decimals,
            identifier,
        })
    }

    /// Retrieves the latest price of this underlying asset
    /// from the chainlink decentralized data feed
    pub async fn latest_round_data(&self) -> Result<Round, AbiError> {
        let (round_id, answer, started_at, updated_at, answered_in_round): (
            u128,
            u128,
            U256,
            U256,
            u128,
        ) = self
            .contract
            .method("latestRoundData", ())?
            .call()
            .await.unwrap();

        // Convert the answer on contract to a string.
        let float_answer: f64 = answer.to_string().parse().unwrap();

        // Convert the contract answer into a human-readable answer
        let human_answer = float_answer / (10f64.powi(self.decimals.into()));

        Ok(Round {
            identifier: self.identifier.to_string(),
            round_id,
            answered_in_round,
            started_at,
            updated_at,
            answer: human_answer,
        })
    }
}

#[cfg(test)]
mod tests {

    use ethers::{abi::Address, providers::Provider};
    use crate::interface::ChainlinkContract;

    #[tokio::test]
    async fn valid_answer() {

        let provider=Provider::try_from("https://bsc-dataseed1.binance.org/").unwrap();

        let chainlink_contract = ChainlinkContract::new(
            &provider,
            "ETH",
            "0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e".parse::<Address>().unwrap(),
        )
        .await
        .unwrap();
        let price_data = chainlink_contract.latest_round_data().await.unwrap();
        println!("Received data: {:#?}", price_data);
        assert!(price_data.answer.ge(&0f64));
    }
}
