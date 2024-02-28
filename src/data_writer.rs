use starknet::core::types::BlockWithTxHashes;
use anyhow::Result;

pub fn write_data(data: Vec<BlockWithTxHashes>, path: &str) -> Result<()> {

    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(&["block_number", "block_timestamp"])?;
    for block in data {
        wtr.serialize(&[format!("{}", block.block_number), format!("{}", block.timestamp)])?;
    }
    wtr.flush()?;

    Ok(())
}