use rand::rngs::StdRng; // TODO: different PRNGs?
use rand::{RngCore, SeedableRng};

use raidpir::client::RaidPirClient;
use raidpir::server::RaidPirServer;
use raidpir::types::RaidPirData;

#[test]
fn test_raidpir_redundancies() {
    let mut prng = StdRng::from_entropy();

    let mut db: Vec<u32> = Vec::with_capacity(256);
    for _i in 0..256 {
        db.push(prng.next_u32());
    }

    for redundancy in 2..=4 {
        let mut servers: Vec<RaidPirServer<u32>> = (0..4)
            .map(|i| RaidPirServer::new(db.clone(), i, 4, redundancy))
            .collect();

        let client = RaidPirClient::new(db.len(), 4, redundancy);

        let seeds = servers.iter_mut().map(|s| s.seed()).collect();

        let queries = client.query(42, &seeds);

        let responses: Vec<u32> = servers
            .iter_mut()
            .zip(seeds.iter().zip(queries.iter()))
            .map(|(server, (seed, query))| server.response(seed.clone(), query))
            .collect();

        assert!(client.combine(responses) == db[42]);
    }
}

#[test]
fn test_large_db() {
    let mut prng = StdRng::from_entropy();

    let mut db: Vec<u32> = Vec::with_capacity(1 << 16);
    for _i in 0..(1u64 << 16) {
        db.push(prng.next_u32());
    }

    let mut servers: Vec<RaidPirServer<u32>> = (0..8)
        .map(|i| RaidPirServer::new(db.clone(), i, 8, 5))
        .collect();

    let client = RaidPirClient::new(db.len(), 8, 5);

    let seeds = servers.iter_mut().map(|s| s.seed()).collect();

    let queries = client.query(1 << 4, &seeds);

    let responses: Vec<u32> = servers
        .iter_mut()
        .zip(seeds.iter().zip(queries.iter()))
        .map(|(server, (seed, query))| server.response(seed.clone(), query))
        .collect();

    assert!(client.combine(responses) == db[1 << 4]);
}

#[test]
fn test_padding() {
    let mut prng = StdRng::from_entropy();

    let mut db: Vec<u32> = Vec::with_capacity(420);
    for _i in 0..420 {
        db.push(prng.next_u32());
    }

    let mut servers: Vec<RaidPirServer<u32>> = (0..4)
        .map(|i| RaidPirServer::new(db.clone(), i, 4, 2))
        .collect();

    let client = RaidPirClient::new(db.len(), 4, 2);

    let seeds = servers.iter_mut().map(|s| s.seed()).collect();

    let queries = client.query(123, &seeds);

    let responses: Vec<u32> = servers
        .iter_mut()
        .zip(seeds.iter().zip(queries.iter()))
        .map(|(server, (seed, query))| server.response(seed.clone(), query))
        .collect();

    assert!(client.combine(responses) == db[123]);
}

#[test]
fn test_bytes() {
    use generic_array::typenum::U8;

    let mut prng = StdRng::from_entropy();

    let mut db: Vec<RaidPirData<U8>> = Vec::with_capacity(256);
    for _i in 0..256 {
        let mut buffer = vec![0; 8];
        prng.fill_bytes(&mut buffer);
        db.push(RaidPirData::from_slice(&buffer));
    }
    db[42] = RaidPirData::from_slice(b"deadbeef");

    let mut servers: Vec<RaidPirServer<RaidPirData<U8>>> = (0..4)
        .map(|i| RaidPirServer::new(db.clone(), i, 4, 2))
        .collect();

    let client = RaidPirClient::new(db.len(), 4, 2);

    let seeds = servers.iter_mut().map(|s| s.seed()).collect();

    let queries = client.query(42, &seeds);

    let responses: Vec<RaidPirData<U8>> = servers
        .iter_mut()
        .zip(seeds.iter().zip(queries.iter()))
        .map(|(server, (seed, query))| server.response(seed.clone(), query))
        .collect();

    let response = client.combine(responses);
    assert!(response.as_slice() == b"deadbeef");
}
