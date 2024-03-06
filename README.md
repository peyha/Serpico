# Starkryo

## A starknet data tool to spam RPCs

Starkryo is a tool made to query large amount of data to Starknet nodes. By specifying a range of blocks and a RPC provider, it should allow fast and efficient fetching of the data.

## Datasets

Currently, the available datasets are

- Blocks

Other datasets will be added ASAP

## How to use

1. Install Rust
2. Use the following command
   `cargo run -- --rpc-url <RPC_URL> --blocks <block_start>:<block_end> --dataset <dataset_name> --path <output_file_path>`

## Improvement list

- Implement idempotence to prevent redundant data querying
- Handle several RPC at once to improve speed
- Use Pyo3 to make the tool usable for Python users
- Add more dataset types to the repo (transactions, traces, opcodes, contracts)
