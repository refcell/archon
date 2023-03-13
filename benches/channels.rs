use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};
use archon::channels::*;

mod harness;

criterion_main!(sync);
criterion_group! {
    name = sync;
    config = Criterion::default().sample_size(10);
    targets =
        bench_channel_tx_data,
}

/// Benchmark message passing between the [Archon] client and the [ChannelManager].
pub fn bench_channel_tx_data(c: &mut Criterion) {
    // let client = harness::mock_archon_client().unwrap();
    // let channel_manager = ChannelManager::new();
    let block_id = harness::await_future(harness::fetch_latest_block_id()).unwrap();
    c.bench_function("tx_data", |b| {
        b.to_async(harness::construct_runtime()).iter(|| async {
            let (tx_bytes, tx_id) = ChannelManager::tx_data(block_id).unwrap();
            assert_eq!(tx_bytes, Bytes::new());
            assert_eq!(tx_id, TransactionID::default());
        })
    });
}
