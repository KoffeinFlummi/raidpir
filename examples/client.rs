use std::time::{Duration, Instant};
use std::net::TcpStream;
use std::io::{Read, Write};

use rayon::iter::*;

use raidpir::client::RaidPirClient;
use raidpir::types::RaidPirData;

fn main() {
    //const DB_SIZE: usize = 1usize << 20;
    const DB_SIZE: usize = 10000;
    const ELEMENT_SIZE: usize = 1024;
    const SERVERS: usize = 2;
    const REDUNDANCY: usize = 2;
    let index = 234;

    let client = RaidPirClient::new(DB_SIZE, SERVERS, REDUNDANCY);

    let addresses = vec!["localhost:3333", "localhost:3334"];

    let mut total: f64 = 0.0;

    for i in 0..100 {
        let t0 = Instant::now();

        // Init connections and retrieve seeds
        let mut streams: Vec<TcpStream> = addresses
            .par_iter() // Establish connections in parallel
            .map(|target| {
                let stream = TcpStream::connect(target).unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
                stream.set_write_timeout(Some(Duration::from_secs(60))).unwrap();
                stream
            })
            .with_max_len(1) // Ensure each iteration gets a thread
            .collect::<Vec<TcpStream>>();

        let t1 = Instant::now();

        // Retrieve seed for each server
        let seeds: Vec<u64> = streams
            .par_iter_mut()
            .map(|stream| {
                let mut seed_bytes = [0; 8];
                stream.read_exact(&mut seed_bytes).unwrap();

                u64::from_le_bytes(seed_bytes)
            })
            .with_max_len(1)
            .collect::<Vec<u64>>();

        let t2 = Instant::now();

        let raidpir_queries = client.query(index, &seeds);

        let t3 = Instant::now();

        // Send queries and retrieve responses
        let responses: Vec<RaidPirData> = streams
            .par_iter_mut()
            .zip(raidpir_queries.par_iter())
            .map(|(stream, raidpir_query)| {
                //println!("{:?}", raidpir_query.as_slice().len());
                stream.write(raidpir_query.as_slice()).unwrap();

                let mut response = Vec::with_capacity(ELEMENT_SIZE);
                stream.read_to_end(&mut response).unwrap();

                RaidPirData::new(response)
            })
            .with_max_len(1)
            .collect();

        let t4 = Instant::now();

        let result = client.combine(responses);

        let t5 = Instant::now();

        //println!("Connection Setup: {:.4}ms", (t1 - t0).as_secs_f32() * 1000.0);
        //println!("Seed Recv: {:.4}ms", (t2 - t1).as_secs_f32() * 1000.0);
        //println!("Query Comp.: {:.4}ms", (t3 - t2).as_secs_f32() * 1000.0);
        //println!("Query Send/Resp Recv: {:.4}ms", (t4 - t3).as_secs_f32() * 1000.0);
        //println!("Resp Comb.: {:.4}ms", (t5 - t4).as_secs_f32() * 1000.0);
        //println!("Total Online Time: {:.4}ms", (t5-t0).as_secs_f64() * 1000.0);

        //assert!(result.as_slice() == b"deadbeef");

        total += (t5 - t0).as_secs_f64() * 1000.0;
    }

    println!("Total Online Time: {:.4}ms", total / 100.0);
}
