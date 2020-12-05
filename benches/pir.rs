use criterion::*;

use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

use raidpir::client::RaidPirClient;
use raidpir::server::RaidPirServer;
use raidpir::util::*;

fn bench_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("Query");
    group.plot_config(PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic));

    for exp in [14, 16, 18, 20, 22].iter() {
        let size = 1usize << exp;

        for threads in [1, 2, 4].iter() {
            group.bench_with_input(BenchmarkId::new(format!("t={}", threads), size), &size, |bench, size| {
                let threadpool = rayon::ThreadPoolBuilder::new().num_threads(*threads).build().unwrap();
                threadpool.install(|| {
                    let mut prng = StdRng::from_entropy();

                    let mut db: Vec<u8> = vec![0; *size];
                    prng.fill_bytes(&mut db);

                    let mut servers: Vec<RaidPirServer<u8>> = (0..2)
                        .map(|i| RaidPirServer::new(db.clone(), i, 2, 2))
                        .collect();

                    let client = RaidPirClient::new(db.len(), 2, 2);

                    let seeds = servers.iter_mut().map(|s| s.seed()).collect();

                    bench.iter(|| {
                        client.query(42, &seeds);
                    });
                });
            });
        }
    }
}

fn bench_xoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("XOR");
    group.plot_config(PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic));

    for exp in [14, 16, 18, 20, 22].iter() {
        let size = 1usize << exp;

        for threads in [1, 2, 4].iter() {
            group.bench_with_input(BenchmarkId::new(format!("t={}", threads), size), &size, |bench, size| {
                let threadpool = rayon::ThreadPoolBuilder::new().num_threads(*threads).build().unwrap();
                threadpool.install(|| {
                    let mut prng = StdRng::from_entropy();

                    let mut a: Vec<usize> = Vec::with_capacity(*size);
                    for _i in 0..*size {
                        a.push(prng.next_u64() as usize);
                    }

                    let b: Vec<usize> = vec![42; *size];

                    bench.iter(|| xor_into_slice(&mut a, &b));
                });
            });
        }
    }
}

criterion_group!(benches, bench_query, bench_xoring);
criterion_main!(benches);
