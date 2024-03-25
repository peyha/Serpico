use std::collections::HashSet;

pub fn split_block_chunks(
    block_start: u64,
    block_end: u64,
    chunk_size: u64,
    seen_chunks: &HashSet<(u64, u64)>,
) -> Vec<(u64, u64)> {
    let mut block_segments = Vec::new();

    for k in (block_start / chunk_size)..(block_end / chunk_size + 1) {
        let left_side = u64::max(block_start, chunk_size * k);
        let right_side = u64::min(chunk_size * k + chunk_size - 1, block_end);
        if !seen_chunks.contains(&(left_side, right_side)) {
            block_segments.push((left_side, right_side));
        }
    }

    block_segments
}
