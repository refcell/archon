use ethers_core::types::{BlockId, BlockNumber};
use ethers_providers::{Http, Middleware, Provider};

use archon::rollup::RollupNode;

/// Requires the following environment variables to be set: ROLLUP_NODE_RPC_URL
#[tokio::test]
async fn test_rollup_node() {
    let rollup_rpc_url = std::env::var("ROLLUP_NODE_RPC_URL").unwrap();
    println!("Rollup node RPC URL: {:?}", rollup_rpc_url);
    // let provider = Provider::<Http>::try_from(&rollup_rpc_url).unwrap();
    // println!("Constructed provider: {:?}", provider);
    // let l1_tip = provider
    //     .get_block(BlockId::Number(BlockNumber::Latest))
    //     .await
    //     .unwrap();
    // println!("L1 tip: {:?}", l1_tip);
    println!("Querying rollup node...");

    // Construct a RollupNode
    let rollup_node = RollupNode::new(&rollup_rpc_url).unwrap();
    let version = rollup_node.version().await.unwrap();
    println!("Rollup node version: {:?}", version);
}
