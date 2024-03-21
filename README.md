# Serpico

## A starknet data tool

Serpico is a tool made to query large amount of data to Starknet nodes. By specifying a range of blocks and a RPC provider, it should allow fast and efficient fetching of the data.

## Datasets

Currently, the available datasets are

- Blocks
- Transactions
- Logs

Other datasets will be added ASAP

## How to use

1. Install Rust
2. Use the following command

```
cargo run -- --rpc-url <RPC_URL> --blocks <block_start>:<block_end> --dataset <dataset_name> --path <output_file_path>
```

## Example

With transactions running

```
cargo run -- --rpc-url $(mesc url blastapi_starknet) --blocks 585084:585085 --dataset transactions --path ./tx.csv
```

The script creates a csv which looks like this

```
block_number,tx_hash,tx_type,tx_type_version,nonce,caller
585084,0x25c244e2f9bd284a95a9e7d66652f9cdb87108e581da64308126bd7ae804f2e,Invoke,V1,97,0x26087dfcee2fbfe6148b3251461a1a4056418cc2b5cf51ec5879b80ad038f9f
585084,0x76f82dc6d07864e55f4af31dbca49ce5539b61ba025054fcbdc03bf95eff02f,Invoke,V1,47,0x166864f51dc742d6e3938292d6735bf1ce060d16c26bfa63e5b06ed941ba6ef
....
```

## Improvement list

- Implement idempotence to prevent redundant data querying
- Handle several RPC at once to improve speed
- Use Pyo3 to make the tool usable for Python users
- Add more dataset types to the repo (traces, opcodes, contracts)

## Thanks

To [Cryo](https://github.com/paradigmxyz/cryo) for inspiration
