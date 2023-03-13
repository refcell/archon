## archon <img src="logo/archon_no_bg.png" width="80" height="18" />

[![build](https://github.com/refcell/archon/actions/workflows/test.yml/badge.svg)](https://github.com/refcell/archon/actions/workflows/test.yml) [![license: MIT](https://img.shields.io/badge/license-MIT-brightgreen.svg)](https://opensource.org/licenses/MIT) [![archon](https://img.shields.io/crates/v/archon.svg)](https://crates.io/crates/archon)

> _archon_ - a "ruler" in Greek; refering to each of the nine chief [magistrates](https://en.wikipedia.org/wiki/Magistrate) in ancient Athens.

`archon` is a maximally efficient, robust batch submission service for the [op-stack](https://stack.optimism.io/) written in pure rust.



## Quickstart

First install `archup`, archon's installer:

```
curl https://raw.githubusercontent.com/refcell/arhcon/main/magup/install | bash
```

To install archon, run `archup`.


## Configuration

The `archon` cli maintains a verbose menu for running a batching service. To see a list of all available commands, run `archon --help`. This will print output similar to the following:

```bash
archon 0.1.0

USAGE:
    archon [OPTIONS]

OPTIONS:
    -d, --data-dir <DATA_DIR>                [default: /Users/user/.archon/data]
    -h, --help                               Print help information
    -n, --network <NETWORK>                  [default: optimism-goerli]
```

Default ports used by `archon`:
- `6061` - pprof
- `7301` - metrics

### Environment Variables

The following environment variables are the default values for `archon`'s configuration.
These can be overridden by setting the environment variable in the shell before running `archon`, or setting the associated flags when running the `archon` cli.

```env
OP_BATCHER_L1_ETH_RPC=http://l1:8545
OP_BATCHER_L2_ETH_RPC=http://l2:8545
OP_BATCHER_ROLLUP_RPC=http://op-node:8545
OP_BATCHER_MIN_L1_TX_SIZE_BYTES=1
OP_BATCHER_MAX_L1_TX_SIZE_BYTES=120000
OP_BATCHER_TARGET_L1_TX_SIZE_BYTES=624
OP_BATCHER_TARGET_NUM_FRAMES=1
OP_BATCHER_APPROX_COMPR_RATIO=1.0
OP_BATCHER_CHANNEL_TIMEOUT=40
OP_BATCHER_POLL_INTERVAL=1s
OP_BATCHER_NUM_CONFIRMATIONS=1
OP_BATCHER_SAFE_ABORT_NONCE_TOO_LOW_COUNT=3
OP_BATCHER_RESUBMISSION_TIMEOUT=30s
OP_BATCHER_MNEMONIC=test test test test test test test test test test test junk
OP_BATCHER_SEQUENCER_HD_PATH="m/44'/60'/0'/0/2"
OP_BATCHER_SEQUENCER_BATCH_INBOX_ADDRESS="${SEQUENCER_BATCH_INBOX_ADDRESS}"
OP_BATCHER_LOG_TERMINAL="true"
OP_BATCHER_PPROF_ENABLED="true"
OP_BATCHER_METRICS_ENABLED="true"
```


## Specifications

`archon`'s channel building process follows several concurrent rules.

- Maximally fill transactions for gas efficiency, while closing channels before they hit consensus timeout or overrun the sequencing window.


Batcher Configuration

The batcher’s channel building process follows several rules at the same time:
make transactions as full as possible for maximum gas efficiency
but don’t keep channels open for too long as to not hit the consensus channel timeout or run over the sequencing window
and optionally limit the overall duration a channel is kept open to provide a better user experience.
Configuration Parameters
It follows a detailed description of all configuration parameters that influence the channel building process.
Gas efficiency
To efficiently fill channels with compressed L2 batch data, the batcher’s channel builder estimates the compressed data size by multiplying the input data size with an approximate compression ratio ApproxComprRatio. This ratio can be set from past experiments and experience, and currently defaults to 0.4.
One batcher transaction contains exactly one frame of maximum size MaxL1TxSizeBytes. The default value is 120_000 bytes because this is close to the maximum data size of transaction p2p gossip messages.
Furthermore, the channel builder can be configured to target channels which span multiple frames by setting the target number of frames TargetNumFrames, which defaults to 1.
The channel builder will, however, actually aim to fill a channel only up to a target L1 transaction size TargetL1TxSizeBytes. The reason for this additional parameter is to have a safety margin between the maximum L1 tx size so that if the resulting compressed output data is just slightly larger than the max L1 tx size, the batcher doesn’t have to send a small left-over frame in a second transaction. When the channel builder then creates the frames from compressed data, these will be filled up to the MaxL1TxSizeBytes, not only to the target size. The default value for the target L1 tx size is 100_000, so 20_000 smaller than the max.
⚠️ Note that these parameters shouldn’t be used to force eager publishing of transactions, e.g. by setting a low target tx size. This will result in suboptimal performance during times of high L2 tx volume and for this reason these parameters should always be set to realistic values, even on testnets. If eager transaction sending is necessary, use the optional parameter maximum channel duration MaxChannelDuration, see section below.


In summary, the default values for these channel builder parameters are
OP_BATCHER_MAX_L1_TX_SIZE_BYTES: 120000
OP_BATCHER_TARGET_L1_TX_SIZE_BYTES: 100000
OP_BATCHER_TARGET_NUM_FRAMES: 1
OP_BATCHER_APPROX_COMPR_RATIO: 0.4



These can safely be used and don’t need to be set explicitly.
Submission Safety Margin
If the batcher always just tried to fill channels up maximally, this would lead to channels running over the consensus channel timeout or missing the sequencing window, resulting in invalid frames being submitted to L1. For this reason, the batcher also watches the consensus channel timeout and sequencing window. Because it would be dangerous to submit the frame transaction just in the very last block before such a timeout, there is a submission safety margin parameter SubSafetyMargin. This is the safety margin, or distance, specified in number of L1 blocks, to keep away from the channel timeout or end of the sequencing window. If the batcher sees L1 approaching closer than the safety margin to these timeouts, it will immediately close the channel and submit frames to L1.
On testnets where quick inclusion is guaranteed, this can be set to a low value, e.g. 8. But on mainnets, this should be set higher to stay confident that the transaction(s) will get safely included on L1.
Note that if this value is set higher or close to the channel timeout duration or SWS, this would effectively lead to channels being closed and sent immediately after creation. If channel duration should be limited, don’t use this parameter but instead the max channel duration, see next section.
Maximum Channel Duration (optional)
In dev- & testnets it is often not desirable to maximally fill txs during times of low L2 transaction volume, because this might result in channels being kept open for several minutes up to (an) hour(s), depending on the consensus channel timeout and SWS.
For this reason, an optional maximum channel duration MaxChannelDuration can be set, measured in L1-blocks. In devnets this could be set as low as 1 to force eager channel closing & submission each polling interval.
⚠️ It cannot be set to 0 because a value of 0 disables this feature.
💡 Note that the effective distance between batcher transactions will be MaxChannelDuration + NumConfirmations because, after sending a transaction, the batcher currently blocks and waits for the transaction to be confirmed for NumConfirmations L1-blocks and only then starts a new channel.
Poll Interval
The poll interval determines how often to query the L2 node for new unsafe L2 blocks. It can safely be set to a low value like 2s (2 seconds). Setting it higher can help reduce traffic between the batcher and its L2 node, but can also result in a possible delay in seeing new L2 blocks and batching them.
Poll Interval History
In the past (until mid January 2023), the batcher’s poll interval determined the channel duration, because in each polling interval, the batcher would poll L2 blocks, open a new channel, put all L2 blocks into it, and then close the channel & submit it as a single frame to L1. The poll interval was the only parameter that influenced channel building.
Now channel building follows above rules, completely independent from the poll interval, because channels can now outlive a single poll interval.
Confirmation Depth
The number of L1-blocks to wait to consider a transaction to be confirmed on L1 can be set with the parameter NumConfirmations. It should be set to the maximum expected reorg depth on L1. In devnets, this might be 1 but should be set higher on real networks.
Worst case, if this is set too low, the batcher could register a channel as being included on L1 but then it gets reorged out and all transactions in that channel would be lost.
Example Configurations
These configurations assume a recent (>= v1.0.0-rc.3) version of the batcher.
Devnet
OP_BATCHER_POLL_INTERVAL: 1s
OP_BATCHER_NUM_CONFIRMATIONS: 1
OP_BATCHER_MAX_CHANNEL_DURATION: 1
OP_BATCHER_SUB_SAFETY_MARGIN: 8
OP_BATCHER_MAX_L1_TX_SIZE_BYTES: 120000
OP_BATCHER_TARGET_L1_TX_SIZE_BYTES: 100000
OP_BATCHER_TARGET_NUM_FRAMES: 1
OP_BATCHER_APPROX_COMPR_RATIO: 0.5

This will lead to eager batch submission each 1(ChannelDuration) + 1(ConfirmationDepth) = 2 L1-blocks. Increase the max. channel duration to a higher value like 5 (1 min with 12 sec L1 block time) or 10 (2 min) if only lower batcher tx frequency is needed.
Goerli
OP_BATCHER_POLL_INTERVAL: 2s
OP_BATCHER_NUM_CONFIRMATIONS: 5
OP_BATCHER_MAX_CHANNEL_DURATION: 5
OP_BATCHER_SUB_SAFETY_MARGIN: 20
OP_BATCHER_MAX_L1_TX_SIZE_BYTES: 120000
OP_BATCHER_TARGET_L1_TX_SIZE_BYTES: 100000
OP_BATCHER_TARGET_NUM_FRAMES: 1
OP_BATCHER_APPROX_COMPR_RATIO: 0.5

This will lead to channels being closed and its frame(s) sent after at most 1 minute (5 * 12 sec L1 block time) plus confirmation depth. Also increased the submission safety margin to 10 which will actually not have any effect with active channel duration limiting, because the SWS and consensus channel timeout minus 10 are still much larger than 5.
Deprecated Parameters
The following parameters are deprecated.
OP_BATCHER_CHANNEL_TIMEOUT — queried from the rollup node.
OP_BATCHER_SEQUENCER_BATCH_INBOX_ADDRESS — queried from the rollup node.
OP_BATCHER_SEQUENCER_GENESIS_HASH — deprecated.
OP_BATCHER_MIN_L1_TX_SIZE_BYTES — conceptually deprecated. Now there’s a target and maximum L1 transaction byte size.
RPC Efficiency
The op-node supports an l1.rpckind flag that customizes the node’s behavior for different L1 infrastructure providers:

Native Geth/Erigon/Nethermind/Parity
Alchemy/QuickNode/Infura

You can see what each l1.rpckind value does here. This is worth experimenting with; it can confer significant cost savings (if using centralized infrastructure) and efficiency savings (if using your own nodes).






## Contributing

All contributions are welcome. Before opening a PR, please submit an issue detailing the bug or feature. When opening a PR, please ensure that your contribution builds on the nightly rust toolchain, has been linted with `cargo fmt`, and contains tests when applicable.

## Disclaimer

_This code is being provided as is. No guarantee, representation or warranty is being made, express or implied, as to the safety or correctness of the code. It has not been audited and as such there can be no assurance it will work as intended, and users may experience delays, failures, errors, omissions or loss of transmitted information. Nothing in this repo should be construed as investment advice or legal advice for any particular facts or circumstances and is not meant to replace competent counsel. It is strongly advised for you to contact a reputable attorney in your jurisdiction for any questions or concerns with respect thereto. The authors is not liable for any use of the foregoing, and users should proceed with caution and use at their own risk._
