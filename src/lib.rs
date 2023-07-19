#![feature(portable_simd)]

use std::simd::{Simd, SimdFloat};

pub fn range_simd(image: &[f32]) -> (f32, f32) {
    const SIMD_WIDTH: usize = 8;
    let (pre, image_simd, post) = image.as_simd();
    let (vec_min, vec_max) = image_simd.iter().fold(
        (
            Simd::from([f32::INFINITY; SIMD_WIDTH]),
            Simd::from([f32::NEG_INFINITY; SIMD_WIDTH]),
        ),
        |(mut min, mut max), &value| {
            min = min.simd_min(value);

            max = max.simd_max(value);

            (min, max)
        },
    );

    let min = vec_min.reduce_min();

    let max = vec_max.reduce_max();
    let (min, max) = min_max_slice(pre, min, max);
    let (min, max) = min_max_slice(post, min, max);

    (min, max)
}

fn min_max_slice(slice: &[f32], min: f32, max: f32) -> (f32, f32) {
    slice.iter().fold((min, max), |(mut min, mut max), value| {
        if value < &min {
            min = *value
        }
        if value > &max {
            max = *value
        }

        return (min, max);
    })
}

//for intrinsics
use core::arch::x86_64::{__m256, _mm256_loadu_ps, _mm256_max_ps, _mm256_min_ps};

pub fn range_simd_intrinsics(image: &[f32]) -> (f32, f32) {
    //load is _mm256_loadu_ps
    //min is _mm256_min_ps
    const SIMD_WIDTH: usize = 8;

    let vec_max = unsafe { _mm256_loadu_ps([f32::NEG_INFINITY; SIMD_WIDTH].as_ptr()) };
    let vec_min = unsafe { _mm256_loadu_ps([f32::INFINITY; SIMD_WIDTH].as_ptr()) };

    let image_chunks = image.chunks_exact(SIMD_WIDTH);
    let remainder = image_chunks.remainder();

    let (vec_min, vec_max) =
        image_chunks.fold((vec_min, vec_max), |(mut min, mut max), value| unsafe {
            let value_vec = _mm256_loadu_ps(value.as_ptr());
            min = _mm256_min_ps(min, value_vec);
            max = _mm256_max_ps(max, value_vec);

            (min, max)
        });

    let vec_min = simd_reg_to_array(vec_min);
    let vec_max = simd_reg_to_array(vec_max);

    let min = vec_min.into_iter().fold(
        f32::INFINITY,
        |min, value| {
            if value < min {
                value
            } else {
                min
            }
        },
    );

    let max = vec_max.into_iter().fold(
        f32::NEG_INFINITY,
        |max, value| {
            if value > max {
                value
            } else {
                max
            }
        },
    );

    // Use the final remainder slice.
    min_max_slice(remainder, min, max)
}

fn simd_reg_to_array(reg: __m256) -> [f32; 8] {
    unsafe { std::mem::transmute(reg) }
}

pub fn range_simd_friendly(image: &[f32]) -> (f32, f32) {
    const SIMD_WIDTH: usize = 8;
    let (vec_min, vec_max) = image.chunks(SIMD_WIDTH).fold(
        ([f32::INFINITY; SIMD_WIDTH], [f32::NEG_INFINITY; SIMD_WIDTH]),
        |(mut min, mut max), value| {
            min.iter_mut().zip(value.iter()).for_each(|(min, &value)| {
                if value < *min {
                    *min = value
                }
            });

            max.iter_mut().zip(value.iter()).for_each(|(max, &value)| {
                if value > *max {
                    *max = value
                }
            });

            (min, max)
        },
    );

    let min = vec_min.into_iter().fold(
        f32::INFINITY,
        |min, value| {
            if value < min {
                value
            } else {
                min
            }
        },
    );

    let max = vec_max.into_iter().fold(
        f32::NEG_INFINITY,
        |max, value| {
            if value > max {
                value
            } else {
                max
            }
        },
    );

    (min, max)
}

// Type your code here, or load an example.
pub fn range(image: &[f32]) -> (f32, f32) {
    image.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(mut min, mut max), value| {
            if value < &min {
                min = *value
            } else {
                min = min
            }

            if value > &max {
                max = *value
            } else {
                max = max
            }

            return (min, max);
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_function(
        image: &[f32],
        expected_min: f32,
        expected_max: f32,
        method: impl Fn(&[f32]) -> (f32, f32),
        name: &str,
    ) {
        let (min, max) = method(image);
        assert_eq!(min, expected_min, "{name} min failed");
        assert_eq!(max, expected_max, "{name} max failed");
    }

    fn run_all_functions(image: &[f32], expected_min: f32, expected_max: f32) {
        run_function(image, expected_min, expected_max, range, "basic");
        run_function(
            image,
            expected_min,
            expected_max,
            range_simd_friendly,
            "simd friendly",
        );
        run_function(image, expected_min, expected_max, range_simd, "simd");
        run_function(
            image,
            expected_min,
            expected_max,
            range_simd_intrinsics,
            "intrinsics",
        );
    }

    #[test]
    fn test_basic_rising() {
        let image = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        run_all_functions(&image[..], 1.0, 8.0);
    }

    #[test]
    fn test_basic_falling() {
        let image = vec![8.0f32, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        run_all_functions(&image[..], 1.0, 8.0);
    }

    #[test]
    fn test_basic_partial_size() {
        let image = vec![8.0f32, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 4.0, 7.0, 10.0];
        run_all_functions(&image[..], 1.0, 10.0);
    }
}
