use crate::Data;
use anyhow::Result;
use starknet::core::types::Transaction;

pub fn write_data(data: Data, path: &str) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;

    match data {
        Data::Blocks(blocks) => {
            wtr.write_record(&[
                "status",
                "block_hash",
                "parent_hash",
                "block_number",
                "new_root",
                "timestamp",
                "sequencer_address",
                "l1_gas_price_in_fri",
                "l1_gas_price_in_wei",
                "starknet_version",
                "tx_count",
            ])?;
            for block in blocks {
                wtr.serialize(&[
                    format!("{:?}", block.status),
                    format!("0x{:x}", block.block_hash),
                    format!("0x{:x}", block.parent_hash),
                    block.block_number.to_string(),
                    format!("0x{:x}", block.new_root),
                    block.timestamp.to_string(),
                    format!("0x{:x}", block.sequencer_address),
                    block.l1_gas_price.price_in_fri.to_string(),
                    block.l1_gas_price.price_in_wei.to_string(),
                    block.starknet_version,
                    block.transactions.len().to_string(),
                ])?;
            }
        }
        Data::Transactions(txs) => {
            wtr.write_record(&["block_number", "tx_hash", "tx_type"])?;
            for tx in txs {
                let tx_type = match tx.0 {
                    Transaction::Invoke(_) => "invoke",
                    Transaction::L1Handler(_) => "l1handler",
                    Transaction::Declare(_) => "declare",
                    Transaction::Deploy(_) => "deploy",
                    Transaction::DeployAccount(_) => "deploy_account",
                };

                wtr.serialize(&[
                    format!("{}", tx.1),
                    format!("{}", tx.0.transaction_hash()),
                    tx_type.to_string(),
                ])?;
            }
        }
        Data::None => (),
    };

    wtr.flush()?;

    Ok(())
}
