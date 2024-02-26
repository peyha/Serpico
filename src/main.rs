use clap::Parser;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::Url ;
use starknet::core::types::BlockId;
use starknet::core::types::MaybePendingBlockWithTxs;
use kdam::tqdm;
use std::io;
use std::fs::File;

mod cli_parser;
use cli_parser::parse_blocks;

mod data_fetcher;
use data_fetcher::fetch_data;

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
}

enum Datasets{
    Blocks,
    // Traces
    // Transactions
    None,
}

#[tokio::main]
async fn main() {

    let args = Cli::parse();

    let stark_client = JsonRpcClient::new(
        HttpTransport::new(
            Url::parse(args.rpc_url.as_str()).unwrap()
        )
    );

    let block_number = stark_client.block_number().await.unwrap();

    let (block_start, block_end) = parse_blocks(args.blocks, block_number).unwrap();

    let dataset = match args.dataset.as_str() {
        "blocks" | "block" => Datasets::Blocks,
        _ => Datasets::None
    };

    // Fetch
    let data = fetch_data(stark_client, dataset, (block_start, block_end)).await;

    // Potentially transform data (remove bad columns, parse types, etc)
    
    // Export data
    let mut wtr = csv::Writer::from_path(args.path.as_str()).unwrap();
    wtr.write_record(&["block_number", "block_timestamp"]).unwrap();
    for block in data {
        wtr.serialize(&[format!("{}", block.block_number), format!("{}", block.timestamp)]).unwrap();
    }
    wtr.flush().unwrap();
}
