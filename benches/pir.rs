use criterion::*;

use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

use raidpir::client::RaidPirClient;
use raidpir::server::RaidPirServer;
use raidpir::util::*;

fn bench_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("Query");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for exp in [20].iter() {
        let size = 1usize << exp;

        for threads in [1, 2, 4].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("t={}", threads), size),
                &size,
                |bench, size| {
                    let threadpool = rayon::ThreadPoolBuilder::new()
                        .num_threads(*threads)
                        .build()
                        .unwrap();
                    threadpool.install(|| {
                        let mut prng = StdRng::from_entropy();

                        let mut db: Vec<u8> = vec![0; *size];
                        prng.fill_bytes(&mut db);

                        let mut servers: Vec<RaidPirServer<u8>> = (0..2)
                            .map(|i| RaidPirServer::new(db.clone(), i, 2, 2, true))
                            .collect();

                        let client = RaidPirClient::new(db.len(), 2, 2);

                        let seeds = servers.iter_mut().map(|s| s.seed()).collect();

                        bench.iter(|| {
                            client.query(42, &seeds);
                        });
                    });
                },
            );
        }
    }
}

fn bench_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("Response");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for exp in [20].iter() {
        let size = 1usize << exp;

        for threads in [1, 2, 4].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("t={}", threads), size),
                &size,
                |bench, size| {
                    let threadpool = rayon::ThreadPoolBuilder::new()
                        .num_threads(*threads)
                        .build()
                        .unwrap();
                    threadpool.install(|| {
                        let mut prng = StdRng::from_entropy();

                        let mut db: Vec<u8> = vec![0; *size];
                        prng.fill_bytes(&mut db);

                        let mut servers: Vec<RaidPirServer<u8>> = (0..2)
                            .map(|i| RaidPirServer::new(db.clone(), i, 2, 2, true))
                            .collect();

                        let client = RaidPirClient::new(db.len(), 2, 2);

                        bench.iter_custom(|iters| {
                            (0..iters)
                                .map(|_| {
                                    let seeds = servers.iter_mut().map(|s| s.seed()).collect();
                                    let queries = client.query(42, &seeds);

                                    let start = std::time::Instant::now();
                                    black_box(servers[0].response(seeds[0], &queries[0]));
                                    start.elapsed()
                                })
                                .sum()
                        });
                    });
                },
            );
        }
    }
}

fn bench_preprocess(c: &mut Criterion) {
    let mut group = c.benchmark_group("Preprocess");
    group
        .sample_size(10)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for exp in [20].iter() {
        let size = 1usize << exp;

        for threads in [1, 2, 4].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("t={}", threads), size),
                &size,
                |bench, size| {
                    let threadpool = rayon::ThreadPoolBuilder::new()
                        .num_threads(*threads)
                        .build()
                        .unwrap();
                    threadpool.install(|| {
                        let mut prng = StdRng::from_entropy();

                        let mut db: Vec<u8> = vec![0; *size];
                        prng.fill_bytes(&mut db);

                        bench.iter_custom(|iters| {
                            (0..iters)
                                .map(|_| {
                                    let server: RaidPirServer<u8> =
                                        RaidPirServer::new(db.clone(), 0, 2, 2, true);

                                    let start = std::time::Instant::now();
                                    black_box(server.preprocess());
                                    start.elapsed()
                                })
                                .sum()
                        });
                    });
                },
            );
        }
    }
}

fn bench_xoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("XOR");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for exp in [20].iter() {
        let size = 1usize << exp;

        for threads in [1, 2, 4].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("t={}", threads), size),
                &size,
                |bench, size| {
                    let threadpool = rayon::ThreadPoolBuilder::new()
                        .num_threads(*threads)
                        .build()
                        .unwrap();
                    threadpool.install(|| {
                        let mut prng = StdRng::from_entropy();

                        let mut a: Vec<u8> = vec![0; *size];
                        prng.fill_bytes(&mut a);

                        let b: Vec<u8> = vec![42; *size];

                        bench.iter(|| xor_into_slice(&mut a, &b));
                    });
                },
            );
        }
    }
}

criterion_group!(benches, bench_query, bench_response, bench_xoring);
//criterion_group!(benches, bench_preprocess);
criterion_main!(benches);
