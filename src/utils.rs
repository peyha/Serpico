use std::collections::HashSet;

pub fn split_block_chunks(
    block_start: u64,
    block_end: u64,
    chunk_size: u64,
    seen_chunks: &mut Vec<(u64, u64)>,
) -> Vec<(u64, u64)> {
    seen_chunks.sort_by(|a, b| {
        if a.0 != b.0 {
            return a.0.cmp(&b.0);
        }
        return b.1.cmp(&a.1);
    });
    let mut seen_chunk_id = 0;

    let mut block_segments = Vec::new();

    for k in (block_start / chunk_size)..(block_end / chunk_size + 1) {
        let left_side = u64::max(block_start, chunk_size * k);

        while (seen_chunk_id < seen_chunks.len() && seen_chunks[seen_chunk_id].0 < left_side) {
            seen_chunk_id += 1;
        }
        let right_side = u64::min(chunk_size * k + chunk_size - 1, block_end);

        if seen_chunk_id < seen_chunks.len() && seen_chunks[seen_chunk_id].0 == left_side {
            // if left side matches, try to extend as much as possible to match current segment
            while (seen_chunk_id + 1 < seen_chunks.len()
                && seen_chunks[seen_chunk_id].1 + 1 == seen_chunks[seen_chunk_id + 1].0
                && seen_chunks[seen_chunk_id + 1].0 <= right_side)
            {
                seen_chunk_id += 1;
            }
            if right_side >= seen_chunks[seen_chunk_id].1 + 1 {
                block_segments.push((seen_chunks[seen_chunk_id].1 + 1, right_side));
            }
        } else {
            block_segments.push((left_side, right_side));
        }
    }

    block_segments
}
