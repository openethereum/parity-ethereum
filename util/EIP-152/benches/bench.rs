use criterion::{Criterion, criterion_group, criterion_main, black_box, Throughput, BenchmarkId};
use eip_152::{avx, portable};
use std::mem;
use std::sync::atomic::{AtomicPtr, Ordering};

type FnRaw = *mut ();
type Blake2bF = fn(&mut [u64; 8], [u64; 16], [u64; 2], bool, usize);

static FN: AtomicPtr<()> = AtomicPtr::new(detect as FnRaw);

fn detect(state: &mut [u64; 8], message: [u64; 16], count: [u64; 2], f: bool, rounds: usize) {
    let fun = if is_x86_feature_detected!("avx2") {
        avx::compress as FnRaw
    } else {
        portable::compress as FnRaw
    };
    FN.store(fun as FnRaw, Ordering::Relaxed);
    unsafe {
        mem::transmute::<FnRaw, Blake2bF>(fun)(state, message, count, f, rounds)
    }
}

pub fn avx_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("avx_impl");

    for rounds in 12..20 {
        group.throughput(Throughput::Elements(rounds as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(rounds),
            &rounds,
            |b, rounds| {
                b.iter(|| {
                    let mut state = [
                        0x6a09e667f2bdc948_u64, 0xbb67ae8584caa73b_u64,
                        0x3c6ef372fe94f82b_u64, 0xa54ff53a5f1d36f1_u64,
                        0x510e527fade682d1_u64, 0x9b05688c2b3e6c1f_u64,
                        0x1f83d9abfb41bd6b_u64, 0x5be0cd19137e2179_u64,
                    ];

                    let message = [
                        0x0000000000636261_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                    ];
                    let count = [3, 0];
                    let f = true;

                    unsafe {
                        let fun = FN.load(Ordering::Relaxed);
                        mem::transmute::<FnRaw, Blake2bF>
                            (fun)
                            (
                                black_box(&mut state),
                                black_box(message),
                                black_box(count),
                                black_box(f),
                                black_box(*rounds)
                            );
                    }
                });
            }
        );
    }

    group.finish();
}



pub fn portable_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("portable_impl");

    for rounds in 12..20 {
        group.throughput(Throughput::Elements(rounds as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(rounds),
            &rounds,
            |b, rounds| {
                b.iter(|| {
                    let mut state = [
                        0x6a09e667f2bdc948_u64, 0xbb67ae8584caa73b_u64,
                        0x3c6ef372fe94f82b_u64, 0xa54ff53a5f1d36f1_u64,
                        0x510e527fade682d1_u64, 0x9b05688c2b3e6c1f_u64,
                        0x1f83d9abfb41bd6b_u64, 0x5be0cd19137e2179_u64,
                    ];

                    let message = [
                        0x0000000000636261_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                        0x0000000000000000_u64, 0x0000000000000000_u64,
                    ];
                    let count = [3, 0];
                    let f = true;

                     portable::compress(
                         black_box(&mut state),
                         black_box(message),
                         black_box(count),
                         black_box(f),
                         black_box(*rounds)
                     );
                });
            }
        );
    }

    group.finish();
}

criterion_group!(benches, avx_benchmark, portable_benchmark);
criterion_main!(benches);