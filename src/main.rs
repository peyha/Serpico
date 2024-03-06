use clap::Parser;
use starknet::core::types::{BlockWithTxHashes, Transaction};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::Url;
use starknet::providers::{JsonRpcClient, Provider};
use std::sync::Arc;

mod cli_parser;
use cli_parser::parse_blocks;

mod data_fetcher;
use data_fetcher::fetch_data;

mod data_writer;
use data_writer::write_data;

mod utils;
use utils::split_block_chunks;

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

    #[arg(short, long, default_value_t = String::from("example.csv"))]
    path: String,

    #[arg(short, long, default_value_t = String::from("csv"))]
    export_type: String,

    #[arg(long, default_value_t = 10000)]
    chunk_size: u64,
}

#[derive(Debug, Clone, Copy)]
enum Datasets {
    Blocks,
    Transactions,
    // Traces
    // Transactions
    None,
}

enum Data {
    Blocks(Vec<BlockWithTxHashes>),
    Transactions(Vec<Transaction>),
    None,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let stark_client = JsonRpcClient::new(HttpTransport::new(
        Url::parse(args.rpc_url.as_str()).unwrap(),
    ));

    let block_number = stark_client.block_number().await.unwrap();

    let (block_start, block_end) = parse_blocks(args.blocks, block_number).unwrap();
    let block_chunks = split_block_chunks(block_start, block_end, args.chunk_size);

    println!("There are {} chunks", block_chunks.len());
    let dataset = match args.dataset.as_str() {
        "blocks" | "block" => Datasets::Blocks,
        "transactions" | "transaction" => Datasets::Transactions,
        _ => Datasets::None,
    };
    // TODO analyze output directory to prevent redundant data downloading

    let rpc_url = Arc::new(args.rpc_url);
    // Fetch
    let mut handles = Vec::new();

    let mut chunk_id = 0;
    for (block_chunk_start, block_chunk_end) in block_chunks {
        let cur_rpc_url = rpc_url.clone();
        let handle = tokio::spawn(async move {
            fetch_data(
                JsonRpcClient::new(HttpTransport::new(
                    Url::parse(cur_rpc_url.as_str()).unwrap(),
                )),
                dataset,
                (block_chunk_start, block_chunk_end),
                chunk_id as u16,
            )
            .await
        });
        handles.push(handle);
        chunk_id += 1;
    }

    let mut merged_data = Vec::new();
    for handle in handles {
        let data_chunk = handle.await.unwrap();

        if let Data::Blocks(data_chunk) = data_chunk {
            merged_data.extend(data_chunk);
        }
    }

    // sort data
    match dataset {
        Datasets::Blocks => merged_data.sort_by_key(|b| b.block_number),
        Datasets::Transactions => todo!(),
        Datasets::None => (),
    };

    let data = match dataset {
        Datasets::Blocks => Data::Blocks(merged_data),
        Datasets::Transactions => todo!(),
        Datasets::None => Data::None,
    };
    //let data = fetch_data(stark_client, dataset, (block_start, block_end)).await;

    // Potentially transform data (remove bad columns, parse types, etc)

    // Export data
    write_data(data, args.path.as_str()).unwrap();
}
