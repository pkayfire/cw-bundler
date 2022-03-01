# CosmWasm CW Bundler

This repo is a CosmWasm smart contract that allows users and other contracts to bundle any combination of `cw20`, `cw721`, `cw1155` tokens under a single `cw721` token.

Disclaimer: This smart contract has not been audited. Please use and fork at your own risk.

## Compatibility

This repo has originally been written to be compatible with the [Terra ecosystem](https://www.terra.money), but can be updated to be compatible with other CosmWasm enabled chains.

- Current CosmWasm version: `0.16.3`

## Using

Contract is live at the below addresses:
- Terra testnet:
- Terra mainnet:

### Execute messages

- `Mint` - Mint a new cw721 bundle.
- `Receive` - Receive and deposit cw20 token sent to the contract into bundle.
- `ReceiveNft` - Receive and deposit cw721 token sent to the contract into bundle.
- `BatchReceive` - Receive and deposit cw1155 token sent to the contract into bundle.
- `Withdraw` - Withdraw all tokens from a bundle.

## Potential use cases

- Allow users to bundle NFTs (cw721) together as collateral and take a loan out on them.
- Allow users to fractionalize assets using cw20 tokens and have the entire asset be represented by a NFT (cw721).
- Allow hierarchical groupings of assets e.g. song NFTs (cw721) under an album NFT (cw721).

## Contributions

- Contributions are welcome, please file an issue or a PR.