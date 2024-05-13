use crate::config::{Configuration, Rustlink};

use super::interface::{ChainlinkContract, PriceData};

/// Retrieves the price of an underlying asset from a particular contract
async fn fetch_round_data_for_contract(
    rustlink_configuration: &Configuration,
    address: &str,
) -> Result<PriceData, alloy::contract::Error> {
    let contract = ChainlinkContract::new(&rustlink_configuration.provider, address).await?;
    contract.latest_round_data().await
}

/// A task that continously fetches cryptocurrency prices from Chainlink decentralised price data feed.
pub async fn fetch_rounds(crypto_prices: Rustlink) {
    let contracts = &crypto_prices.configuration.contracts;
    loop {
        for contract_configuration in contracts {
            let symbol = &contract_configuration.0;
            let address = &contract_configuration.1;
            match fetch_round_data_for_contract(&crypto_prices.configuration, address).await
            .and_then(|price_data| Ok({
                let _ =crypto_prices.tree.insert(
                    symbol,
                    bincode::serialize::<PriceData>(&price_data).unwrap(),
                );

            })).or_else(|error| Err(error)) {
                Ok(())=>{},
                Err(error)=>{
                    log::error!("Failed updating price: {}",error);
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(
                crypto_prices.configuration.fetch_interval_seconds,
            ))
            .await;
        }

        // Give back control to tokio runtime.
        tokio::time::sleep(std::time::Duration::from_secs(
            crypto_prices.configuration.fetch_interval_seconds,
        ))
        .await;
    }
}
