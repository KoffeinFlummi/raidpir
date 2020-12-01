#![feature(test)]

extern crate test;
use test::Bencher;

use rand::rngs::StdRng;
use rand::{SeedableRng, RngCore};

use raidpir::client::RaidPirClient;
use raidpir::server::RaidPirServer;

#[bench]
fn bench_query(b: &mut Bencher) {
    let mut prng = StdRng::from_entropy();

    let mut db: Vec<u32> = Vec::with_capacity(1 << 22);
    for _i in 0..(1 << 22) {
        db.push(prng.next_u32());
    }

    let mut servers: Vec<RaidPirServer<u32>> = (0..4)
        .map(|i| RaidPirServer::new(db.clone(), i, 4, 3))
        .collect();

    let client = RaidPirClient::new(db.len(), 4, 3);

    let seeds = servers.iter_mut().map(|s| s.seed()).collect();

    b.iter(|| {
        client.query(42, &seeds);
    });
}
