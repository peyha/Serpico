use clap::Parser;
use polars::frame::DataFrame;
use polars::prelude::*;
use starknet::core::types::{
    BlockStatus, BlockWithTxHashes, EmittedEvent, InvokeTransaction, Transaction,
};
use starknet::core::types::{DeclareTransaction, DeployAccountTransaction};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::Url;
use starknet::providers::{JsonRpcClient, Provider};
use std::collections::{BTreeMap, HashSet};
use std::fs::{read_dir, File};
use std::sync::Arc;
use tokio::sync::Semaphore;

mod cli_parser;
use cli_parser::parse_blocks;

mod data_fetcher;
use data_fetcher::fetch_data;

mod data_writer;
use data_writer::write_data;

mod utils;
use utils::split_block_chunks;

mod error;
use error::SerpicoError;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    // RPC API Provider
    #[arg(short, long)]
    rpc_url: String,

    // Block interval to use
    #[arg(short, long)]
    blocks: String,

    // Dataset to fetch
    #[arg(short, long)]
    dataset: String,

    //TODO add parameters
    #[arg(short, long, default_value_t = String::from("all"))]
    columns: String,

    #[arg(short, long, default_value_t = String::from("."))]
    path: String,

    #[arg(short, long, default_value_t = String::from("csv"))]
    export_type: String,

    #[arg(short, long, default_value_t = 4)]
    max_concurrent_chunk: u64,

    #[arg(long, default_value_t = 10000)]
    chunk_size: u64,
}

#[derive(Debug, Clone, Copy)]
enum Datasets {
    Blocks,
    Transactions,
    Logs,
    // Traces
    // Transactions
    None,
}

impl Datasets {
    pub fn to_name(self) -> &'static str {
        match self {
            Datasets::Blocks => "blocks",
            Datasets::Transactions => "transactions",
            Datasets::Logs => "logs",
            Datasets::None => "",
        }
    }
}
enum Data {
    Blocks(Vec<BlockWithTxHashes>),
    Transactions(Vec<(Transaction, u64)>),
    Logs(Vec<EmittedEvent>),
    None,
}

impl Data {
    pub fn to_dataframe(self) -> DataFrame {
        let mut columns = BTreeMap::new();

        match self {
            Data::Blocks(blocks) => {
                for block in blocks {
                    columns
                        .entry("status")
                        .or_insert(vec![])
                        .push(format!("{:?}", block.status));
                    columns
                        .entry("block_hash")
                        .or_insert(vec![])
                        .push(format!("0x{:x}", block.block_hash));
                    columns
                        .entry("parent_hash")
                        .or_insert(vec![])
                        .push(format!("0x{:x}", block.parent_hash));
                    columns
                        .entry("block_number")
                        .or_insert(vec![])
                        .push(block.block_number.to_string());
                    columns
                        .entry("new_root")
                        .or_insert(vec![])
                        .push(format!("0x{:x}", block.new_root));
                    columns
                        .entry("timestamp")
                        .or_insert(vec![])
                        .push(block.timestamp.to_string());
                    columns
                        .entry("sequencer_address")
                        .or_insert(vec![])
                        .push(format!("0x{:x}", block.sequencer_address));
                    columns
                        .entry("l1_gas_price_in_fri")
                        .or_insert(vec![])
                        .push(block.l1_gas_price.price_in_fri.to_string());
                    columns
                        .entry("l1_gas_price_in_wei")
                        .or_insert(vec![])
                        .push(block.l1_gas_price.price_in_wei.to_string());
                    columns
                        .entry("starknet_version")
                        .or_insert(vec![])
                        .push(block.starknet_version);
                    columns
                        .entry("tx_count")
                        .or_insert(vec![])
                        .push(block.transactions.len().to_string());
                }
            }
            Data::Transactions(txs) => {
                for (tx, block_number) in txs {
                    columns
                        .entry("block_number")
                        .or_insert(vec![])
                        .push(block_number.to_string());
                    columns
                        .entry("transaction_hash")
                        .or_insert(vec![])
                        .push(format!("0x{:x}", tx.transaction_hash()));

                    let (tx_type, version, nonce, caller) = match tx {
                        Transaction::Invoke(InvokeTransaction::V0(_)) => (
                            "Invoke".to_string(),
                            "V0".to_string(),
                            "None".to_string(),
                            "None".to_string(),
                        ),
                        Transaction::Invoke(InvokeTransaction::V1(sub_tx)) => (
                            "Invoke".to_string(),
                            "V1".to_string(),
                            sub_tx.nonce.to_string(),
                            format!("0x{:x}", sub_tx.sender_address),
                        ),
                        Transaction::Invoke(InvokeTransaction::V3(sub_tx)) => (
                            "Invoke".to_string(),
                            "V3".to_string(),
                            sub_tx.nonce.to_string(),
                            format!("0x{:x}", sub_tx.sender_address),
                        ),
                        Transaction::L1Handler(sub_tx) => (
                            "L1Handler".to_string(),
                            sub_tx.version.to_string(),
                            sub_tx.nonce.to_string().clone(),
                            "None".to_string(),
                        ),
                        Transaction::Declare(DeclareTransaction::V0(sub_tx)) => (
                            "Declare".to_string(),
                            "V0".to_string(),
                            "None".to_string(),
                            format!("0x{:x}", sub_tx.sender_address),
                        ),
                        Transaction::Declare(DeclareTransaction::V1(sub_tx)) => (
                            "Declare".to_string(),
                            "V1".to_string(),
                            sub_tx.nonce.to_string(),
                            format!("0x{:x}", sub_tx.sender_address),
                        ),
                        Transaction::Declare(DeclareTransaction::V2(sub_tx)) => (
                            "Declare".to_string(),
                            "V2".to_string(),
                            sub_tx.nonce.to_string(),
                            format!("0x{:x}", sub_tx.sender_address),
                        ),
                        Transaction::Declare(DeclareTransaction::V3(sub_tx)) => (
                            "Declare".to_string(),
                            "V3".to_string(),
                            sub_tx.nonce.to_string(),
                            format!("0x{:x}", sub_tx.sender_address),
                        ),
                        Transaction::Deploy(sub_tx) => (
                            "Deploy".to_string(),
                            sub_tx.version.to_string(),
                            "None".to_string(),
                            "None".to_string(),
                        ),
                        Transaction::DeployAccount(DeployAccountTransaction::V1(sub_tx)) => (
                            "DeployAccount".to_string(),
                            "V1".to_string(),
                            sub_tx.nonce.to_string(),
                            "None".to_string(),
                        ),
                        Transaction::DeployAccount(DeployAccountTransaction::V3(sub_tx)) => (
                            "DeployAccount".to_string(),
                            "V3".to_string(),
                            sub_tx.nonce.to_string(),
                            "None".to_string(),
                        ),
                    };

                    columns.entry("tx_type").or_insert(vec![]).push(tx_type);
                    columns
                        .entry("tx_type_version")
                        .or_insert(vec![])
                        .push(version);

                    columns
                        .entry("nonce")
                        .or_insert(vec![])
                        .push(nonce.to_string());
                    columns
                        .entry("caller")
                        .or_insert(vec![])
                        .push(caller.to_string());
                }
            }
            Data::Logs(logs) => {
                for event in logs {
                    columns
                        .entry("block_number")
                        .or_insert(vec![])
                        .push(event.block_number.unwrap_or(0).to_string());
                    columns
                        .entry("tx_hash")
                        .or_insert(vec![])
                        .push(format!("0x{:x}", event.transaction_hash));
                    columns
                        .entry("contract_address")
                        .or_insert(vec![])
                        .push(format!("0x{:x}", event.from_address));
                    columns.entry("keys").or_insert(vec![]).push(format!(
                        "{:?}",
                        event.keys.iter().map(|x| format!("0x{:x}", x))
                    ));
                    columns.entry("data").or_insert(vec![]).push(format!(
                        "{:?}",
                        event.data.iter().map(|x| format!("0x{:x}", x))
                    ));
                }
            }
            Data::None => (),
        };
        DataFrame::new(
            columns
                .into_iter()
                .map(|(name, values)| Series::new(name, values))
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }
}
#[tokio::main]
async fn main() -> Result<(), SerpicoError> {
    let args = Cli::parse();

    let dataset = match args.dataset.as_str() {
        "blocks" | "block" => Datasets::Blocks,
        "transactions" | "transaction" => Datasets::Transactions,
        "logs" | "events" | "log" => Datasets::Logs,
        _ => Datasets::None,
    };

    let stark_client = JsonRpcClient::new(HttpTransport::new(
        Url::parse(args.rpc_url.as_str()).map_err(SerpicoError::UrlParsingErr)?,
    ));

    let block_number = stark_client
        .block_number()
        .await
        .map_err(SerpicoError::ClientErr)?;

    let (block_start, block_end) = parse_blocks(args.blocks, block_number)?;

    let mut chunks_seen: HashSet<(u64, u64)> = HashSet::new();

    if let Ok(entries) = read_dir(args.path.clone()) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let chunks = file_name
                    .to_str()
                    .unwrap()
                    .split(|c| c == '_' || c == '.')
                    .collect::<Vec<&str>>();

                if chunks.len() == 6
                    && chunks[0] == dataset.to_name()
                    && chunks[1] == "from"
                    && chunks[3] == "to"
                    && chunks[5] == args.export_type
                {
                    if let Ok(block_start) = chunks[2].parse::<u64>() {
                        if let Ok(block_end) = chunks[4].parse::<u64>() {
                            chunks_seen.insert((block_start, block_end));
                        }
                    }
                }
            }
        }
    }
    let block_chunks = split_block_chunks(block_start, block_end, args.chunk_size, &chunks_seen);

    println!("There are {} chunks", block_chunks.len());

    // TODO analyze output directory to prevent redundant data downloading

    let rpc_url = Arc::new(args.rpc_url);
    let path = Arc::new(args.path);
    // Fetch
    let semaphore = Arc::new(Semaphore::new(args.max_concurrent_chunk as usize));
    let mut handles = Vec::new();

    let mut chunk_id = 0;
    for (block_chunk_start, block_chunk_end) in block_chunks {
        let cur_rpc_url = rpc_url.clone();
        let cur_path = path.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let handle = tokio::spawn(async move {
            let res = fetch_data(
                JsonRpcClient::new(HttpTransport::new(
                    Url::parse(cur_rpc_url.as_str()).map_err(SerpicoError::UrlParsingErr)?,
                )),
                dataset,
                (block_chunk_start, block_chunk_end),
                chunk_id as u16,
            )
            .await;
            let mut dataframe = res.unwrap().to_dataframe();

            let file_name = format!(
                "{}/{}_from_{}_to_{}.csv",
                cur_path,
                dataset.to_name(),
                block_chunk_start,
                block_chunk_end
            );

            let mut file = File::create(file_name.as_str()).unwrap();
            CsvWriter::new(&mut file).finish(&mut dataframe).unwrap();
            drop(permit);
            Ok((block_chunk_start, block_chunk_end))
        });
        handles.push(handle);
        chunk_id += 1;
    }

    for handle in handles {
        let _: Result<(u64, u64), SerpicoError> = handle.await.unwrap();
    }

    Ok(())
}
