use crate::Data;
use anyhow::Result;

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
        Data::Transactions(txs) => todo!(),
        Data::None => (),
    };

    wtr.flush()?;

    Ok(())
}
