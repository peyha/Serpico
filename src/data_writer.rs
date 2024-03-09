use crate::Data;
use anyhow::Result;
use starknet::core::types::Transaction;

pub fn write_data(data: Data, path: &str) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;

    match data {
        Data::Blocks(blocks) => {
            wtr.write_record(&["block_number", "block_timestamp"])?;
            for block in blocks {
                wtr.serialize(&[
                    format!("{}", block.block_number),
                    format!("{}", block.timestamp),
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
