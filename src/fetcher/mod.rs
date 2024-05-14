use super::interface::{ChainlinkContract, Round};
use crate::config::Reflector::Sender;
use crate::config::{Configuration, Rustlink};

/// Retrieves the price of an underlying asset from a particular contract
async fn fetch_round_data_for_contract(
    rustlink_configuration: &Configuration,
    identifier: &str,
    address: &str,
) -> Result<Round, alloy::contract::Error> {
    let contract =
        ChainlinkContract::new(&rustlink_configuration.provider, identifier, address).await?;
    contract.latest_round_data().await
}

// The function signature looks good, but ensure all types (Rustlink, Round, etc.) are properly defined.
pub async fn fetch_rounds(rustlink: Rustlink) {
    let contracts = &rustlink.configuration.contracts;

    // This loop runs indefinitely, fetching price data.
    loop {
        for contract_configuration in contracts {
            let identifier = &contract_configuration.0; // This variable wasn't used in your original code.
            let address = &contract_configuration.1;

            // Fetch price data and attempt to send it via the channel.
            match fetch_round_data_for_contract(&rustlink.configuration, identifier, address).await
            {
                Ok(price_data) => {
                    match rustlink.reflector {
                        Sender(ref sender) => {
                            // Attempt to send the PriceData through the channel.
                            if let Err(error) = sender.send(price_data).await {
                                log::error!("Failed sending data: {}", error);
                            }
                        }
                    }
                }
                Err(error) => {
                    log::error!("Failed updating price: {}", error);
                }
            }

            // Wait for the specified interval before fetching the next price.
            tokio::time::sleep(std::time::Duration::from_secs(
                rustlink.configuration.fetch_interval_seconds,
            ))
            .await;
        }

        // Depending on your intention, you might not need this additional sleep here because
        // the loop itself already waits after each contract is fetched, which might be enough.
        // If you wish all contracts to be fetched at once and then wait, keep this.
        // Otherwise, it might cause longer than expected delays between fetches.
    }
}
