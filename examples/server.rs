use std::time::Instant;
use std::net::TcpListener;
use std::io::{Read, Write};

use bitvec::vec::BitVec;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

use raidpir::server::RaidPirServer;
use raidpir::types::RaidPirData;

fn main() {
    //const DB_SIZE: usize = 1usize << 20;
    const DB_SIZE: usize = 10000;
    const ELEMENT_SIZE: usize = 1024;
    const SERVERS: usize = 2;
    const REDUNDANCY: usize = 2;
    let index = 234;

    let id: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let port: usize = std::env::args().nth(2).unwrap().parse().unwrap();

    let mut prng = StdRng::from_seed([0x44; 32]);
    let mut db: Vec<Vec<u8>> = Vec::with_capacity(DB_SIZE);
    for _i in 0..DB_SIZE {
        let mut buffer = vec![0; ELEMENT_SIZE];
        prng.fill_bytes(&mut buffer);
        db.push(buffer);
    }
    db[index] = b"deadbeef".to_vec();

    let raidpir_db: Vec<RaidPirData> = db.iter().map(|x| RaidPirData::new(x.clone())).collect();

    let server = RaidPirServer::new(raidpir_db, id, SERVERS, REDUNDANCY, true);
    server.preprocess();

    let addr = format!("localhost:{}", port);
    let listener = TcpListener::bind(addr).unwrap();

    println!("Listening on {:?}...", listener.local_addr().unwrap());

    let mut query: Vec<u8> = vec![0; DB_SIZE / SERVERS / 8];
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let seed = server.seed();
                stream.write(&seed.clone().to_le_bytes()).unwrap();

                stream.read_exact(&mut query).unwrap();

                let bitvec = BitVec::from_vec(query.clone());
                let t0 = Instant::now();
                let response = server.response(seed, &bitvec);
                println!("Response Comp.: {:.4}ms", t0.elapsed().as_secs_f64() * 1000.0);

                stream.write(response.as_slice()).unwrap();
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}
