pub mod rustlink;
mod error;
mod fetcher;
mod interface;
#[cfg(test)]
mod tests {

    use async_std::channel::unbounded;

    use crate::rustlink::{Reflector, Rustlink};

    #[tokio::test]
    async fn ensure_price_is_received() {
        let mut contracts: Vec<(String, String)> = Vec::new();
        contracts.push((
            "ETH".to_string(),
            "0x9ef1B8c0E4F7dc8bF5719Ea496883DC6401d5b2e".to_string(),
        ));

        let (sender, receiver) = unbounded();

        let rustlink = Rustlink::try_new(
            "https://bsc-dataseed1.binance.org/",
            1,
            Reflector::Sender(sender),
            contracts,
        )
        .unwrap();

        rustlink.start();
        let round_data = receiver.recv().await.unwrap();
        println!("Received data: {:#?}", round_data);
        assert!(round_data.answer.ge(&0f64));
    }
}
