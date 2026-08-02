use crate::features::dct::apply_dct2;

/// Calculates a perceptual hash for a 32x32 black-and-white matrix
pub fn p_hash(matrix: [[u8; 32]; 32]) -> i64 {
    // Apply DCT to matrix
    let dct = apply_dct2(&matrix);

    // Crop the matrix to 8x8 and calculate the median value
    let mut vals = Vec::with_capacity(63);
    for row in 0..8 {
        for col in 0..8 {
            if col == 0 && row == 0 {
                continue;
            }
            vals.push(dct[row][col]);
        }
    }
    vals.sort_by(|a, b| a.partial_cmp(b).expect("Cannot sort pHash values"));
    let median = vals[vals.len() / 2];

    // Calculating the hash
    let mut hash = 0_i64;
    for row in 0..8 {
        for col in 0..8 {
            if row == 0 && col == 0 {
                continue;
            }
            hash <<= 1;
            if dct[row][col] > median {
                hash |= 1;
            }
        }
    }
    hash
}

/// Calculates the aHash for 64 image pixels (8x8)
pub fn a_hash(pixels: [u8; 64]) -> i64 {
    let avg = pixels.iter().map(|p| *p as u32).sum::<u32>() / pixels.len() as u32;
    let mut hash = 0_i64;
    for (idx, p) in pixels.into_iter().enumerate() {
        if p as u32 > avg {
            hash |= 1 << idx;
        }
    }
    hash
}
