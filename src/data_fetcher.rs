use crate::Datasets;
use kdam::tqdm;
use starknet::core::types::{BlockId, BlockWithTxHashes, MaybePendingBlockWithTxHashes};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

pub async fn fetch_data(
    client: JsonRpcClient<HttpTransport>,
    dataset: Datasets,
    (block_start, block_end): (u64, u64),
) -> Vec<BlockWithTxHashes> {
    match dataset {
        Datasets::Blocks => fetch_blocks(client, (block_start, block_end)).await,
        Datasets::None => Vec::new(),
    }
}

pub async fn fetch_blocks(
    client: JsonRpcClient<HttpTransport>,
    (block_start, block_end): (u64, u64),
) -> Vec<BlockWithTxHashes> {
    let mut data = Vec::new();
    for block in tqdm!(block_start..(block_end + 1)) {
        match client
            .get_block_with_tx_hashes(BlockId::Number(block))
            .await
            .unwrap()
        {
            MaybePendingBlockWithTxHashes::Block(b) => data.push(b),
            MaybePendingBlockWithTxHashes::PendingBlock(_) => (),
        };
    }

    data
}
