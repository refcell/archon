use ethers_core::types::{
    BlockId,
    BlockNumber,
};
use ethers_providers::{
    Http,
    Middleware,
    Provider,
};

/// Requires the following environment variables to be set:
/// - L1_RPC_URL
#[tokio::test]
async fn test_l1_client() {
    let l1_rpc_url = std::env::var("L1_RPC_URL").unwrap();
    let provider = Provider::<Http>::try_from(l1_rpc_url).unwrap();
    println!("Constructed provider: {:?}", provider);
    let l1_tip = provider
        .get_block(BlockId::Number(BlockNumber::Latest))
        .await
        .unwrap();
    println!("L1 tip: {:?}", l1_tip);
}
