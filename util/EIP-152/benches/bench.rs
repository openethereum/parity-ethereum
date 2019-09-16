use rustc_hex::FromHex;
use criterion::{Criterion, criterion_group, criterion_main};

use eip_152::{avx, portable};

struct TestVector {
    state: [u64; 8],
    message: [u64; 16],
    count: [u64; 2],
    f: bool,
    rounds: usize,
    expected: [u64; 8]
}

impl TestVector {
    fn from_string(input: &str, output: &str) -> Self {
        let bytes: Vec<u8> = input.from_hex().unwrap();

        let mut state = [0u64; 8];
        let mut message = [0u64; 16];
        let mut count = [0u64; 2];

        let rounds = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let f = match bytes[212] {
            1 => true,
            0 => false,
            _ => unreachable!()
        };

        to_u64_slice(&bytes[4..68], &mut state);
        to_u64_slice(&bytes[68..196], &mut message);
        to_u64_slice(&bytes[196..212], &mut count);

        let output: Vec<u8> = output.from_hex().unwrap();
        let mut expected = [0u64; 8];
        to_u64_slice(&output[..], &mut expected);

        Self {
            state,
            message,
            count,
            f,
            rounds,
            expected,
        }
    }

    fn to_parts(self) -> ([u64; 8], [u64; 16], [u64; 2], bool, usize, [u64; 8]) {
        let Self {
            state,
            message,
            count,
            f,
            rounds,
            expected
        } = self;

        (state, message, count, f, rounds, expected)
    }
}

fn to_u64_slice(vec: &[u8], slice: &mut [u64]) {
    vec.chunks(8).enumerate().for_each(|(index, val)| {
        slice[index] = u64::from_le_bytes([val[0], val[1], val[2], val[3], val[4], val[5], val[6], val[7]])
    })
}

pub fn avx_benchmark(c: &mut Criterion) {
    c.bench_function(
        "avx_impl",
        move |b| {
            b.iter(move || {
                let vectors = vec![
                    TestVector::from_string(
                        "0000000048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e\
            511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b6162630000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000300000000000000000000000000000001",
                        "08c9bcf367e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d282e6ad7f520e511f6c3e\
            2b8c68059b9442be0454267ce079217e1319cde05b",
                    ).to_parts(),
                    TestVector::from_string(
                        "0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e\
            511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b6162630000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000300000000000000000000000000000001",
                        "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5\
            de4533cc9518d38aa8dbf1925ab92386edd4009923",
                    ).to_parts(),
                    TestVector::from_string(
                        "0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e\
            511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b6162630000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000300000000000000000000000000000000",
                        "75ab69d3190a562c51aef8d88f1c2775876944407270c42c9844252c26d2875298743e7f6d5ea2f2d3e8d2\
            26039cd31b4e426ac4f2d3d666a610c2116fde4735",
                    ).to_parts(),
                    TestVector::from_string(
                        "0000000148c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e\
            511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b6162630000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000300000000000000000000000000000001",
                        "b63a380cb2897d521994a85234ee2c181b5f844d2c624c002677e9703449d2fba551b3a8333bcdf5f2f7e0\
            8993d53923de3d64fcc68c034e717b9293fed7a421",
                    ).to_parts()
                ];

                for (mut state, message, count, f, rounds, expected) in vectors {
                    if is_x86_feature_detected!("avx2") {
                        unsafe {
                            avx::compress(&mut state, message, count, f, rounds);
                        }
                    }
                    assert_eq!(state, expected);
                }
            })
        }
    );
}

pub fn portable_benchmark(c: &mut Criterion) {
    c.bench_function(
        "portable_impl",
        move |b| {
            b.iter(move || {
                let vectors = vec![
                    TestVector::from_string(
                        "0000000048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e\
            511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b6162630000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000300000000000000000000000000000001",
                        "08c9bcf367e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d282e6ad7f520e511f6c3e\
            2b8c68059b9442be0454267ce079217e1319cde05b",
                    ).to_parts(),
                    TestVector::from_string(
                        "0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e\
            511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b6162630000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000300000000000000000000000000000001",
                        "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5\
            de4533cc9518d38aa8dbf1925ab92386edd4009923",
                    ).to_parts(),
                    TestVector::from_string(
                        "0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e\
            511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b6162630000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000300000000000000000000000000000000",
                        "75ab69d3190a562c51aef8d88f1c2775876944407270c42c9844252c26d2875298743e7f6d5ea2f2d3e8d2\
            26039cd31b4e426ac4f2d3d666a610c2116fde4735",
                    ).to_parts(),
                    TestVector::from_string(
                        "0000000148c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e\
            511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b6162630000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000300000000000000000000000000000001",
                        "b63a380cb2897d521994a85234ee2c181b5f844d2c624c002677e9703449d2fba551b3a8333bcdf5f2f7e0\
            8993d53923de3d64fcc68c034e717b9293fed7a421",
                    ).to_parts()
                ];

                for (mut state, message, count, f, rounds, expected) in vectors {
                    portable::compress(&mut state, message, count, f, rounds);
                    assert_eq!(state, expected);
                }
            })
        }
    );
}

criterion_group!(benches, avx_benchmark, portable_benchmark);
criterion_main!(benches);