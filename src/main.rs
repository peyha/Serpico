use clap::Parser;
use starknet::core::types::BlockWithTxHashes;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::Url;
use starknet::providers::{JsonRpcClient, Provider};

mod cli_parser;
use cli_parser::parse_blocks;

mod data_fetcher;
use data_fetcher::fetch_data;

mod data_writer;
use data_writer::write_data;

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

enum Datasets {
    Blocks,
    // Traces
    // Transactions
    None,
}

enum Data {
    Blocks(Vec<BlockWithTxHashes>),
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

    let dataset = match args.dataset.as_str() {
        "blocks" | "block" => Datasets::Blocks,
        _ => Datasets::None,
    };
    // TODO analyze output directory to prevent redundant data downloading

    // Fetch
    let data = fetch_data(stark_client, dataset, (block_start, block_end)).await;

    // Potentially transform data (remove bad columns, parse types, etc)

    // Export data
    write_data(data, args.path.as_str()).unwrap();
}
