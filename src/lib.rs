mod fetcher;
mod interface;
mod error;
pub mod config;

#[cfg(test)]
mod tests {

    use std::{fs, path::Path};

    use crate::config::Rustlink;
    fn remove_test_db(db_path: &str) {
        if Path::new(db_path).exists() {
            fs::remove_dir_all(db_path).expect("Failed to remove test database");
        }
    }
    #[tokio::test]
    async fn ensure_price_is_received() {
        let mut contracts: Vec<(String, String)> = Vec::new();
        contracts.push((
            "ETH".to_string(),
            "0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e".to_string(),
        ));
        let crypto_prices = Rustlink::try_new(
            "https://bsc-dataseed1.binance.org/",
            1,
            "./test-crypto-prices",
            contracts,
        ).unwrap();

        crypto_prices.fetch();

        // Within 10 seconds we can confidently check for price
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let btc_price = crypto_prices.get_answer("ETH").unwrap();
        println!("Received data: {:#?}", btc_price);
        remove_test_db("./test-crypto-prices");
        assert!(btc_price.price.ge(&0f64));
    }
}
