use crate::{Data, SerpicoError};
use anyhow::Result;
use starknet::core::types::{
    DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Transaction,
};

pub fn write_data(data: Data, path: &str) -> Result<(), SerpicoError> {
    let mut wtr = csv::Writer::from_path(path).map_err(SerpicoError::WriterErr)?;

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
            ])
            .map_err(SerpicoError::WriterErr)?;
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
                ])
                .map_err(SerpicoError::WriterErr)?;
            }
        }
        Data::Transactions(txs) => {
            // todo handle custom tx type info
            wtr.write_record(&[
                "block_number",
                "tx_hash",
                "tx_type",
                "tx_type_version",
                "nonce",
                "caller",
            ])
            .map_err(SerpicoError::WriterErr)?;
            for (tx, block_number) in txs {
                let record = match tx {
                    Transaction::Invoke(InvokeTransaction::V0(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "Invoke".to_string(),
                        "V0".to_string(),
                        "None".to_string(),
                        "None".to_string(),
                    ],
                    Transaction::Invoke(InvokeTransaction::V1(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "Invoke".to_string(),
                        "V1".to_string(),
                        sub_tx.nonce.to_string(),
                        format!("0x{:x}", sub_tx.sender_address),
                    ],
                    Transaction::Invoke(InvokeTransaction::V3(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "Invoke".to_string(),
                        "V3".to_string(),
                        sub_tx.nonce.to_string(),
                        format!("0x{:x}", sub_tx.sender_address),
                    ],
                    Transaction::L1Handler(sub_tx) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "L1Handler".to_string(),
                        sub_tx.version.to_string(),
                        sub_tx.nonce.to_string(),
                        "None".to_string(),
                    ],
                    Transaction::Declare(DeclareTransaction::V0(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "Declare".to_string(),
                        "V0".to_string(),
                        "None".to_string(),
                        format!("0x{:x}", sub_tx.sender_address),
                    ],
                    Transaction::Declare(DeclareTransaction::V1(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "Declare".to_string(),
                        "V1".to_string(),
                        sub_tx.nonce.to_string(),
                        format!("0x{:x}", sub_tx.sender_address),
                    ],
                    Transaction::Declare(DeclareTransaction::V2(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "Declare".to_string(),
                        "V2".to_string(),
                        sub_tx.nonce.to_string(),
                        format!("0x{:x}", sub_tx.sender_address),
                    ],
                    Transaction::Declare(DeclareTransaction::V3(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "Declare".to_string(),
                        "V1".to_string(),
                        sub_tx.nonce.to_string(),
                        format!("0x{:x}", sub_tx.sender_address),
                    ],
                    Transaction::Deploy(sub_tx) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "Deploy".to_string(),
                        sub_tx.version.to_string(),
                        "None".to_string(),
                        "None".to_string(),
                    ],
                    Transaction::DeployAccount(DeployAccountTransaction::V1(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "DeployAccount".to_string(),
                        "V1".to_string(),
                        sub_tx.nonce.to_string(),
                        "None".to_string(),
                    ],
                    Transaction::DeployAccount(DeployAccountTransaction::V3(sub_tx)) => [
                        block_number.to_string(),
                        format!("0x{:x}", sub_tx.transaction_hash),
                        "DeployAccount".to_string(),
                        "V3".to_string(),
                        sub_tx.nonce.to_string(),
                        "None".to_string(),
                    ],
                };

                wtr.serialize(&record).map_err(SerpicoError::WriterErr)?;
            }
        }
        Data::Logs(logs) => {
            wtr.write_record(&[
                "block_number",
                "tx_hash",
                "contract_address",
                "keys",
                "data",
            ])
            .map_err(SerpicoError::WriterErr)?;
            for log in logs {
                wtr.serialize(&[
                    log.block_number.unwrap_or(0).to_string(),
                    format!("0x{:x}", log.transaction_hash),
                    format!("0x{:x}", log.from_address),
                    format!("{:?}", log.keys),
                    format!("{:?}", log.data),
                ])
                .map_err(SerpicoError::WriterErr)?;
            }
        }
        Data::None => (),
    };

    wtr.flush().map_err(SerpicoError::IoErr)?;

    Ok(())
}
