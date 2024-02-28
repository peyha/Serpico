use crate::{Data, Datasets};
use kdam::tqdm;
use starknet::core::types::{BlockId, MaybePendingBlockWithTxHashes};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

pub async fn fetch_data(
    client: JsonRpcClient<HttpTransport>,
    dataset: Datasets,
    (block_start, block_end): (u64, u64),
) -> Data {
    match dataset {
        Datasets::Blocks => fetch_blocks(client, (block_start, block_end)).await,
        Datasets::None => Data::None,
    }
}

pub async fn fetch_blocks(
    client: JsonRpcClient<HttpTransport>,
    (block_start, block_end): (u64, u64),
) -> Data {
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

    Data::Blocks(data)
}
