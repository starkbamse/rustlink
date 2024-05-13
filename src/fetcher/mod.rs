use super::{
    interface::{ChainlinkContract, PriceData, Serializable},
    Configuration, CryptoPrices,
};

/// Retrieves the price of an underlying asset from a particular contract
async fn fetch_price_for_contract(
    rustlink_configuration: Configuration,
    address: &str,
) -> Result<PriceData, alloy::contract::Error> {
    let contract = ChainlinkContract::new(rustlink_configuration.provider, address).await?;
    contract.get_price().await
}

/// A task that continously fetches cryptocurrency prices from Chainlink decentralised price data feed.
pub async fn fetch_prices(crypto_prices: CryptoPrices) {
    let contracts = &crypto_prices.configuration.contracts;
    loop {
        for contract_configuration in contracts {
            let symbol = &contract_configuration.0;
            let address = &contract_configuration.1;
            match fetch_price_for_contract(crypto_prices.clone().configuration, address).await {
                Ok(price_data) => {
                    match crypto_prices
                        .tree
                        .insert(symbol, price_data.to_bin().unwrap())
                    {
                        Ok(_) => {}
                        Err(error) => {
                            log::error!("Could not update price in db: {}", error);
                        }
                    }
                }
                Err(error) => {
                    log::error!("Could not fetch price: {}", error);
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
