use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image_max_min_test::{range, range_simd, range_simd_friendly, range_simd_intrinsics};
use rand::distributions::{Distribution, Uniform};

fn criterion_benchmark(c: &mut Criterion) {
    let size = 512 * 512;
    let between = Uniform::from(0..4095);
    let mut rng = rand::thread_rng();
    let image: Vec<f32> = (0..size).map(|_| between.sample(&mut rng) as f32).collect();

    c.bench_with_input(BenchmarkId::new("Image Range", size), &image, |b, image| {
        b.iter(|| range(&image[..]))
    });

    c.bench_with_input(
        BenchmarkId::new("Image Range SIMD Friendly", size),
        &image,
        |b, image| b.iter(|| range_simd_friendly(&image[..])),
    );

    c.bench_with_input(
        BenchmarkId::new("Image Range SIMD", size),
        &image,
        |b, image| b.iter(|| range_simd(&image[..])),
    );

    c.bench_with_input(
        BenchmarkId::new("Image Range Instrinsics", size),
        &image,
        |b, image| b.iter(|| range_simd_intrinsics(&image[..])),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
