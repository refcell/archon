<!--  <img src="logo/archon_no_bg.png" width="80" height="18" /> -->

<img width="100%" src="logo/background.png">

<!-- # archon  ⚖ -->

[![build](https://github.com/refcell/archon/actions/workflows/test.yml/badge.svg)](https://github.com/refcell/archon/actions/workflows/test.yml) [![license: MIT](https://img.shields.io/badge/license-MIT-brightgreen.svg)](https://opensource.org/licenses/MIT) [![archon](https://img.shields.io/crates/v/archon.svg)](https://crates.io/crates/archon)

`archon` is a maximally efficient, robust batch submission service for the [op-stack](https://stack.optimism.io/) written in pure rust.

## Quickstart

First install `archup`, archon's installer:

```
curl https://raw.githubusercontent.com/refcell/arhcon/main/archup/install | bash
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

They can be overridden by setting the environment variable in the shell before running `archon`, or setting the associated flags when running the `archon` cli.

```env
OP_BATCHER_L1_ETH_RPC=http://l1:8545
OP_BATCHER_L2_ETH_RPC=http://l2:8545
OP_BATCHER_ROLLUP_RPC=http://op-node:8545
OP_BATCHER_MAX_L1_TX_SIZE_BYTES=120000
OP_BATCHER_TARGET_L1_TX_SIZE_BYTES=624
OP_BATCHER_TARGET_NUM_FRAMES=1
OP_BATCHER_APPROX_COMPR_RATIO=1.0
OP_BATCHER_POLL_INTERVAL=1s
OP_BATCHER_NUM_CONFIRMATIONS=1
OP_BATCHER_SAFE_ABORT_NONCE_TOO_LOW_COUNT=3
OP_BATCHER_RESUBMISSION_TIMEOUT=30s
OP_BATCHER_MNEMONIC=test test test test test test test test test test test junk
OP_BATCHER_SEQUENCER_HD_PATH="m/44'/60'/0'/0/2"
OP_BATCHER_LOG_TERMINAL="true"
OP_BATCHER_PPROF_ENABLED="true"
OP_BATCHER_METRICS_ENABLED="true"
```

## Specifications

`archon`'s channel building process follows several concurrent rules.

- Maximally fill transactions for gas efficiency
- Close channels before they hit consensus timeout or overrun the sequencing window.
- Optionally, we also want to limit the duration of channels to provide a better data availability user experience.

Below, we outline the parameters that effect the channel building process.

#### Gas Efficiency

The batcher's channel builder estimates the compressed data size by multiplying the data size input with an approximate compression ratio, `ApproxComprRatio`. This enables the channel builder to efficiently fill channels with _compressed_ L2 batch data and is set empirically. The current default value is `0.4`.

One batcher transaction contains exactly one frame of maximum size, `MaxL1TxSizeBytes`. This is set to `120_000` by default which is as close to the maximum size of p2p gossip transaction messages as possible.

The channel builder can be further configured to target channels which span multiple frames by setting the target number of frames, `TargetNumFrames`. This defaults to 1.

In actuality, the channel builder will aim to fill a channel only up to `TargetL1TxSizeBytes`, a target L1 transaction size. The reason for this additional parameter is to have a safety margin between the maximum L1 transaction size so that if the resulting compressed output data is just slightly larger than the max L1 tx size, the batcher doesn’t have to send a small left-over frame in a second transaction. When the channel builder then creates the frames from compressed data, these will be filled up to the `MaxL1TxSizeBytes`, not only to the target size. The default value for the target L1 tx size is `100_000`, so `20_000` smaller than the max (ie `MaxL1TxSizeBytes`).

> **Warning**
>
> Note that these parameters shouldn’t be used to force eager publishing of transactions. For example, by setting a low target tx size.
> This will result in suboptimal performance during times of high L2 tx volume and so these parameters should always be set to realistic values, even on testnets.
> If eager transaction sending is necessary, use the optional parameter maximum channel duration `MaxChannelDuration`, see section below.

In summary, the default values for these channel builder parameters are:
`OP_BATCHER_MAX_L1_TX_SIZE_BYTES`: `120000`
`OP_BATCHER_TARGET_L1_TX_SIZE_BYTES`: `100000`
`OP_BATCHER_TARGET_NUM_FRAMES`: `1`
`OP_BATCHER_APPROX_COMPR_RATIO`: `0.4`

These can safely be used and don’t need to be set explicitly.

#### Submission Safety Margin

If the batcher always tried to fill channels up maximally, this would lead to channels running over the channel timeout or missing the sequencing window, resulting in invalid frames being submitted to L1.

For this reason, the batcher also watches the channel timeout and sequencing window.

Because it would be dangerous to submit the frame transaction just in the very last block before such a timeout, there is a submission safety margin parameter called `SubSafetyMargin`. This is the safety margin, or distance, specified in number of L1 blocks, from the channel timeout or end of the sequencing window. If the L1 block number advances into the safety margin, the batcher will immediately close the channel and submit frames to L1.

On testnets where quick inclusion is guaranteed, this can be set to a lower value (e.g. 8). But on mainnets, this should be set higher to stay confident that the transaction(s) will get safely included on L1.

> **Note**
>
> If this value is set higher or close to the channel timeout duration or SWS, this would effectively lead to channels being closed and sent immediately after creation. If channel duration should be limited, don’t use this parameter but instead the max channel duration, see next section.

#### Maximum Channel Duration (optional)

In devnets and testnets it is often not desirable to maximally fill transactions during times of low L2 transaction volume. The reason being that this might result in channels being kept open for several minutes up to (an) hour(s), depending on the consensus channel timeout and SWS.

In order to bound this behavior, the batcher exposes an optional maximum channel duration parameter, `MaxChannelDuration`, measured in L1 blocks. In devnets this could be set as low as 1 to force eager channel closing and submission on each polling interval.

> **Warning**
>
> The max channel duration should never be set to 0 because a value of 0 disables this feature.

> **Note**
>
> The effective distance between batcher transactions will be `MaxChannelDuration` + `NumConfirmations` (the number of L1 block confirmations).
> This is because after sending a transaction the batcher blocks and waits for the transaction to be confirmed for `NumConfirmations` L1 blocks and then starts a new channel.

#### Poll Interval

The batcher's poll interval determines how often to query the L2 node for new _unsafe_ L2 blocks. It can safely be set to a low value like `2s` (2 seconds). Setting it higher can help reduce traffic between the batcher and its L2 node, but can also result in a possible delay in seeing new L2 blocks and batching them.

Until recently, the batcher’s poll interval determined the channel duration. In each polling interval, the batcher would poll for L2 blocks, open a new channel putting all those L2 blocks into it, and then close and submit the channel as a single frame to L1. As such, the poll interval was the only parameter that influenced channel building.

Now, channel building follows the rules outlined above which are fully independent from the poll interval. This allows channels to outlive a single poll interval, separating L2 node latency from channel building.

#### Confirmation Depth

As noted above, the number of L1 block confirmations to wait to consider a transaction finalized can be set with the parameter, `NumConfirmations`.

`NumConfirmations` should be set to the maximum expected reorg depth on L1. Making this too low can result in transactions being reorged out of L1 and the associated channel being completely lost. On the other hand, making this too high would result in batcher stalling, and could even lead to a channel timeout.

In devnets, the `NumConfirmations` might be 1 but should be set quite a bit higher on mainnets.

#### Example Configurations

**Devnet**

This configuration makes the batcher submit transactions eagerly, roughly every `2` L1 blocks. That is, the `ChannelDuration` (`1`) + `NumConfirmations` (`1`) = `2` L1 blocks. To decrease the batch submission frequency, the `MaxChannelDuration` should be increased to a higher value like `5` (1 min with 12 sec L1 block time) or `10` (2 min).

```bash
OP_BATCHER_POLL_INTERVAL: 1s
OP_BATCHER_NUM_CONFIRMATIONS: 1
OP_BATCHER_MAX_CHANNEL_DURATION: 1
OP_BATCHER_SUB_SAFETY_MARGIN: 8
OP_BATCHER_MAX_L1_TX_SIZE_BYTES: 120000
OP_BATCHER_TARGET_L1_TX_SIZE_BYTES: 100000
OP_BATCHER_TARGET_NUM_FRAMES: 1
OP_BATCHER_APPROX_COMPR_RATIO: 0.5
```

**Goerli**


This will lead to channels being closed and its frame(s) sent after at most 1 minute (5 * 12 sec L1 block time) plus confirmation depth. Also increased the submission safety margin to 10 which will actually not have any effect with active channel duration limiting, because the SWS and consensus channel timeout minus 10 are still much larger than 5.

```bash
OP_BATCHER_POLL_INTERVAL: 2s
OP_BATCHER_NUM_CONFIRMATIONS: 5
OP_BATCHER_MAX_CHANNEL_DURATION: 5
OP_BATCHER_SUB_SAFETY_MARGIN: 20
OP_BATCHER_MAX_L1_TX_SIZE_BYTES: 120000
OP_BATCHER_TARGET_L1_TX_SIZE_BYTES: 100000
OP_BATCHER_TARGET_NUM_FRAMES: 1
OP_BATCHER_APPROX_COMPR_RATIO: 0.5
```

#### Other Parameters

A few other parameters are worth mentioning which are either deprecated or queried from the rollup node.

Both the `OP_BATCHER_CHANNEL_TIMEOUT` and `OP_BATCHER_SEQUENCER_BATCH_INBOX_ADDRESS` are now queried from the rollup node, which is specified via the `OP_BATCHER_ROLLUP_RPC` (see [Environment Variables](#-Environment-Variables)).

The `OP_BATCHER_SEQUENCER_GENESIS_HASH` is deprecated. And the `OP_BATCHER_MIN_L1_TX_SIZE_BYTES` is now calculated differently as described in [Gas Effeciency](#-Gas-Efficiency) section above.

## FAQ

> Why doesn't the op-batcher currently send multiple frames per transaction?

The optimal strategy with respect to gas efficiency is to send exactly one large frame per transaction. This minimizes overhead while maximizing the amount of data per transaction, amortizing the fixed 21k gas costs of L1 transactions.

We could even let a channel span multiple full transactions, to achieve an even higher compression ratio (using parameter `TargetNumFrames`), because the more data per compression buffer, the better it can overall compress. But there is a tradeoff here where channels would be posted less frequently to the data availability layer. Realistically the gains are probably little here, so having one full frame per transaction is our best empirical guess for balancing gas efficient with UX (user experience).

> **Note**
>
> The protocol still allows to have multiple frames per batcher tx to allow for future optimizations like
>
> - If there's a small leftover frame, we can actually start a new channel and construct a transaction with both the leftover frame and the next channels first frame. This is a _greedy_ approach at filling up transactions in order to amortize the fixed gas costs of L1 transactions.
> - If we build multiple channels in parallel, we can submit multiple frames in a single transaction.

## Why "Archon"?

The term "archon" means "ruler" in Greek. In ancient Athens, an "Archon" refers to each of the nine chief [magistrates](https://en.wikipedia.org/wiki/Magistrate), civilians who were elected to administer the law. Like the magistrates, Archon reigns judicial and executive power. Responsible for the state of the "rollup", Archon batches and submits chain data to a data availability layer.

## Contributing

All contributions are welcome. Before opening a PR, please submit an issue detailing the bug or feature. When opening a PR, please ensure that your contribution builds on the nightly rust toolchain, has been linted with `cargo fmt`, and contains tests when applicable.

## Disclaimer

_This code is being provided as is. No guarantee, representation or warranty is being made, express or implied, as to the safety or correctness of the code. It has not been audited and as such there can be no assurance it will work as intended, and users may experience delays, failures, errors, omissions or loss of transmitted information. Nothing in this repo should be construed as investment advice or legal advice for any particular facts or circumstances and is not meant to replace competent counsel. It is strongly advised for you to contact a reputable attorney in your jurisdiction for any questions or concerns with respect thereto. The authors is not liable for any use of the foregoing, and users should proceed with caution and use at their own risk._
