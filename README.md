# Rust Simd Comparison

This is a test repository to compare Rust SIMD options on an array min/max function which is problematic for automatic vectorization.

I see using the portable SIMD methods around 4x faster than the basic implementation.

The intrinsics method is around 8x the basic implementation

## Compile and Run

This requires nightly rust to run the portable SIMD methods.

I also run with the rustflags for AVX2 and optimisation level 3 to max everything out on my system!

```
RUSTFLAGS="-C target-feature=+avx2 -C opt-level=3" cargo bench 
```

## Methods

The basic naive method should be pretty self explainatory. This doesn't vectorise well since we have a 1 element width dependency between iterations.

I attempted a SIMD friendly version which works on 8 elements in parallel at a time which shows a speed up in some cases but was limited.

range_simd uses the portable SIMD methods on nightly. This was a significant improvement over other versions.

Finally `range_simd_intrinsics` calls directly to the x86 AVX instrinsics. Surprisingly this was double the speed of the `range_simd`` method. Looking in Godbolt `range_simd` uses some additional calls in min/max which seem unnecessary in our case which is the difference in performance.