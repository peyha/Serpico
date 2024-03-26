use clap::Parser;
use polars::frame::DataFrame;
use starknet::core::types::{BlockWithTxHashes, EmittedEvent, Transaction};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::Url;
use starknet::providers::{JsonRpcClient, Provider};
use std::collections::HashSet;
use std::fs::read_dir;
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
        match self {
            Data::Blocks(block) => {}
            Data::Transactions(txs) => {}
            Data::Logs(logs) => {}
            Data::None => {}
        }
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
            let file_name = format!(
                "{}/{}_from_{}_to_{}.csv",
                cur_path,
                dataset.to_name(),
                block_chunk_start,
                block_chunk_end
            );
            let _ = write_data(res.unwrap(), file_name.as_str());
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
