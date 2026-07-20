#![allow(clippy::needless_range_loop)]

use std::{f32::consts::PI, sync::LazyLock};

/// Dimensionality of Discrete Cosine Transform matrices
const N: usize = 32;

/// LazyLock Basis for Discrete Cosine Transform
static BASIS: LazyLock<[[f32; N]; N]> = LazyLock::new(|| {
    let mut b = [[0.0; N]; N];
    for i in 0..N {
        for n in 0..N {
            let basis = (((2 * n + 1) as f32) * i as f32 * PI) / (2 * N) as f32;
            b[i][n] = basis.cos();
        }
    }
    b
});

/// LazyLock Alpha for Discrete Cosine Transform
static ALPHA: LazyLock<[f32; N]> = LazyLock::new(|| {
    let mut a = [0f32; N];
    for i in 0..N {
        let alpha = if i == 0 {
            (1f32 / N as f32).sqrt()
        } else {
            (2f32 / N as f32).sqrt()
        };
        a[i] = alpha;
    }
    a
});

/// Applies the DCT-II (Discrete Cosine Transform II) to the matrix
///
/// Read more here:
/// [Discrete_Cosine_Transform](https://en.wikipedia.org/wiki/Discrete_cosine_transform#DCT-II)
pub fn apply_dct2(matrix: &[[u8; N]; N]) -> [[f32; N]; N] {
    let mut dct = [[0f32; N]; N];
    for row in 0..N {
        dct[row] = dct_1d(&matrix[row].map(|i| (i as f32) / 255.0));
    }
    for col in 0..N {
        let mut tmp = [0f32; N];
        for row in 0..N {
            tmp[row] = dct[row][col];
        }
        let tmp = dct_1d(&tmp);
        for row in 0..N {
            dct[row][col] = tmp[row];
        }
    }
    dct
}

/// Applies the DCT-II transform to a one-dimensional row
fn dct_1d(ik: &[f32; N]) -> [f32; N] {
    let mut dct = [0f32; N];
    for i in 0..N {
        let mut sum = 0f32;
        for n in 0..N {
            let sig = ik[n] * BASIS[i][n];
            sum += sig;
        }
        dct[i] = ALPHA[i] * sum;
    }
    dct
}
