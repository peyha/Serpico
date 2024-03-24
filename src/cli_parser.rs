use crate::SerpicoError;

pub fn parse_blocks(block_str: String, last_block: u64) -> Result<(u64, u64), SerpicoError> {
    let block_parts: Vec<&str> = block_str.split(':').collect();
    let block_start = match block_parts[0] {
        "" => 1,
        x => x.parse::<u64>().map_err(SerpicoError::IntParsingErr)?,
    };

    let block_end = match block_parts[1] {
        "" | "latest" => last_block,
        x => x.parse::<u64>().map_err(SerpicoError::IntParsingErr)?,
    };

    //TODO block requirements
    Ok((block_start, block_end))
}
