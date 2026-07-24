use crate::features::dct::apply_dct2;

/// Calculates a perceptual hash for a 32x32 black-and-white matrix
pub fn p_hash(matrix: [[u8; 32]; 32]) -> i64 {
    // Apply DCT to matrix
    let dct = apply_dct2(&matrix);

    // Crop the matrix to 8x8 and calculate the average value
    let mut summ = 0.0;
    let mut count = 0;

    for j in 0..8 {
        let y = dct[j];
        for i in 0..8 {
            if i == 0 && j == 0 {
                continue;
            }
            count += 1;
            summ += y[i];
        }
    }
    let avg = summ / count as f32;

    // Calculating the hash
    let mut hash = 0_i64;
    for row in 0..8 {
        for col in 0..8 {
            if row == 0 && col == 0 {
                continue;
            }
            hash <<= 1;
            if dct[col][row] > avg {
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
