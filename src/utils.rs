pub fn split_block_chunks(block_start: u64, block_end: u64, chunk_size: u64) -> Vec<(u64, u64)> {
    let mut block_segments = Vec::new();

    let mut cur_block = block_start;

    while cur_block <= block_end {
        let right_side = u64::min(cur_block + chunk_size - 1, block_end);
        block_segments.push((cur_block, right_side));
        cur_block += chunk_size;
    }

    block_segments
}
