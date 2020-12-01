//! Methods for preprocessing and responding to RAID-PIR queries.

use std::collections::HashMap;
use std::ops::BitXorAssign;

use bitvec::prelude::*;
use rand::rngs::StdRng; // TODO: different PRNGs?
use rand::{RngCore, SeedableRng};

use crate::util::*;

const QUEUE_SIZE: usize = 32;

/**
 * RaidPir server.
 *
 * T is the type of database elements, and needs to be bit-xor-assignable
 * and have a default value, i.e. integer types.
 *
 * When not using integer values, care needs to be taken to ensure that all
 * values have the same size, and that T::default() returns an object of
 * that size. See [crate::types::RaidPirData].
 */
#[derive(Debug)]
pub struct RaidPirServer<T> {
    db: Vec<T>,
    servers: usize,
    redundancy: usize,
    queue: HashMap<u64, T>,
    queue_used: HashMap<u64, T>,
}

impl<T: Clone + Default + BitXorAssign> RaidPirServer<T> {
    /**
     * Create a new server object and prepare the database.
     */
    pub fn new(mut db: Vec<T>, id: usize, servers: usize, redundancy: usize) -> Self {
        // TODO: move to param type?, store unpadded size

        // pad databse to next multiple of (servers * 64)
        if db.len() % (servers * 64) != 0 {
            db.resize_with(
                db.len() + (servers * 64) - (db.len() % (servers * 64)),
                Default::default,
            )
        }

        assert!(db.len() % (servers * 64) == 0);
        assert!(redundancy >= 2 && redundancy <= servers);

        let blocks_per_server = db.len() / servers;
        db.rotate_left(id * blocks_per_server);

        Self {
            db,
            servers,
            redundancy,
            queue: HashMap::with_capacity(QUEUE_SIZE),
            queue_used: HashMap::new(),
        }
    }

    /**
     * Preprocess queries by preparing a queue of seeds and partial answers.
     */
    pub fn preprocess(&mut self) {
        let blocks_per_server = self.db.len() / self.servers;

        let mut rng = StdRng::from_entropy();

        while self.queue.len() < QUEUE_SIZE {
            let seed = rng.next_u64();

            let random_bits = rand_bitvec(seed, blocks_per_server * (self.redundancy - 1));

            let mut preprocessed = T::default();
            for (i, r) in random_bits.iter().enumerate() {
                if !r {
                    continue;
                }

                preprocessed ^= self.db[blocks_per_server + i].clone();
            }

            self.queue.insert(seed, preprocessed);
        }
    }

    /**
     * Return a seed from the queue.
     */
    pub fn seed(&mut self) -> u64 {
        // TODO: mutex?

        if self.queue.len() == 0 {
            self.preprocess();
        }

        let seed = *self.queue.keys().next().unwrap();

        self.queue_used
            .insert(seed, self.queue.remove(&seed).unwrap());

        seed
    }

    /**
     * Calculate response to the given query with the given seed.
     */
    pub fn response(&mut self, seed: u64, query: &BitVec) -> T {
        let mut answer = self.queue_used.remove(&seed).unwrap(); // TODO

        for (i, q) in query.iter().enumerate() {
            if !q {
                continue;
            }

            answer ^= self.db[i].clone();
        }

        answer
    }
}
