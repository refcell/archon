#!/bin/bash

archon \
  --proposer-address "0x87A159604e2f18B01a080F672ee011F39777E640" \
  --batcher-inbox "0xff00000000000000000000000000000000042069" \
  --l2-client-rpc-url $L2_RPC_URL \
  --l1-client-rpc-url $L1_RPC_URL \
  --data-availability-layer mainnet \
  --network optimism \
  --polling-interval 5 \
  --sequencer-private-key "0xa0bba68a40ddd0b573c344de2e7dd597af69b3d90e30a87ec91fa0547ddb6ab8" \
  --proposer-private-key "0x4a6e5ceb37cd67ed8e740cc25b0ee6d11f6cfabe366daad1c908dec1d178bc72" \
  --batcher-address "0x87A159604e2f18B01a080F672ee011F39777E640" \
  --sequencer-address "0xf4031e0983177452c9e7F27f46ff6bB9CA5933E1" \
  --batcher-private-key "0x4a6e5ceb37cd67ed8e740cc25b0ee6d11f6cfabe366daad1c908dec1d178bc72"
