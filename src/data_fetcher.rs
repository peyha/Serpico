use crate::{Data, Datasets};
use kdam::tqdm;
use starknet::core::types::{BlockId, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

pub async fn fetch_data(
    client: JsonRpcClient<HttpTransport>,
    dataset: Datasets,
    (block_start, block_end): (u64, u64),
    chunk_id: u16,
) -> Data {
    match dataset {
        Datasets::Blocks => fetch_blocks(client, (block_start, block_end), chunk_id).await,
        Datasets::Transactions => fetch_txs(client, (block_start, block_end), chunk_id).await,
        Datasets::None => Data::None,
    }
}

pub async fn fetch_blocks(
    client: JsonRpcClient<HttpTransport>,
    (block_start, block_end): (u64, u64),
    chunk_id: u16,
) -> Data {
    let mut data = Vec::new();
    for block in tqdm!(
        block_start..(block_end + 1),
        desc = format!("block {} to {}", block_start, block_end),
        position = chunk_id
    ) {
        let mut continuer = true;
        while continuer {
            continuer = false;
            match client
                .get_block_with_tx_hashes(BlockId::Number(block))
                .await
            {
                Ok(MaybePendingBlockWithTxHashes::Block(b)) => data.push(b),
                Ok(MaybePendingBlockWithTxHashes::PendingBlock(_)) => (),
                Err(_) => continuer = true,
            };
        }
    }

    Data::Blocks(data)
}

pub async fn fetch_txs(
    client: JsonRpcClient<HttpTransport>,
    (block_start, block_end): (u64, u64),
    chunk_id: u16,
) -> Data {
    let mut data = Vec::new();
    for block in tqdm!(
        block_start..(block_end + 1),
        desc = format!("block {} to {}", block_start, block_end),
        position = chunk_id
    ) {
        let mut continuer = true;
        while continuer {
            continuer = false;
            match client.get_block_with_txs(BlockId::Number(block)).await {
                Ok(MaybePendingBlockWithTxs::Block(b)) => {
                    data.extend(b.transactions);
                }
                Ok(MaybePendingBlockWithTxs::PendingBlock(_)) => (),
                Err(_) => continuer = true,
            };
        }
    }

    Data::Transactions(data)
}
