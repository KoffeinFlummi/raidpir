//! Methods for preprocessing and responding to RAID-PIR queries.

use std::collections::HashMap;
use std::ops::BitXor;
use std::ops::BitXorAssign;
use std::sync::RwLock;

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
    russians: Option<Vec<Vec<T>>>,
    servers: usize,
    redundancy: usize,
    queue: RwLock<HashMap<u64, T>>,
    queue_used: RwLock<HashMap<u64, T>>,
}

impl<T: Clone + Default + BitXor<Output=T> + BitXorAssign> RaidPirServer<T> {
    /**
     * Create a new server object and prepare the database.
     */
    pub fn new(mut db: Vec<T>, id: usize, servers: usize, redundancy: usize, russians: bool) -> Self {
        // TODO: move to param type?, store unpadded size

        // pad databse to next multiple of (servers * 8)
        if db.len() % (servers * 8) != 0 {
            db.resize_with(
                db.len() + (servers * 8) - (db.len() % (servers * 8)),
                Default::default,
            )
        }

        assert!(db.len() % (servers * 8) == 0);
        assert!(redundancy >= 2 && redundancy <= servers);

        let blocks_per_server = db.len() / servers;
        db.rotate_left(id * blocks_per_server);

        let russians = russians.then(|| db[0..blocks_per_server].chunks(8).map(|chunk| {
            (0..=255).map(|i| {
                BitVec::<Lsb0,u8>::from_vec(vec![i])
                    .iter()
                    .zip(chunk)
                    .filter(|(q, _)| **q)
                    .fold(T::default(), |a, (_, b)| a ^ b.clone())
            }).collect()
        }).collect());

        Self {
            db,
            russians,
            servers,
            redundancy,
            queue: RwLock::new(HashMap::with_capacity(QUEUE_SIZE)),
            queue_used: RwLock::new(HashMap::new()),
        }
    }

    /**
     * Preprocess queries by preparing a queue of seeds and partial answers.
     */
    pub fn preprocess(&self) {
        let blocks_per_server = self.db.len() / self.servers;

        let mut rng = StdRng::from_entropy();

        loop {
            let seed = rng.next_u64();
            let random_bits = rand_bitvec(seed, blocks_per_server * (self.redundancy - 1));

            let mut db_iter = self.db.iter();
            // TODO: replace with advance_by once that is stable
            for _i in 0..blocks_per_server {
                db_iter.next().unwrap();
            }

            let preprocessed = random_bits
                .iter()
                .zip(db_iter)
                .filter(|(q, _)| **q)
                .fold(T::default(), |a, (_, b)| a ^ b.clone());

            let mut queue = self.queue.write().unwrap();
            queue.insert(seed, preprocessed);
            if queue.len() >= QUEUE_SIZE {
                break;
            }
        }
    }

    /**
     * Return a seed from the queue.
     */
    pub fn seed(&self) -> u64 {
        let len = {
            let queue = self.queue.read().unwrap();
            queue.len()
        };

        if len == 0 {
            self.preprocess();
        }

        let mut queue = self.queue.write().unwrap();
        let mut queue_used = self.queue_used.write().unwrap();

        let seed = *queue.keys().next().unwrap();

        queue_used.insert(seed, queue.remove(&seed).unwrap());

        seed
    }

    /**
     * Calculate response to the given query with the given seed.
     */
    pub fn response(&self, seed: u64, query: &BitVec<Lsb0, u8>) -> T {
        let mut answer = {
            let mut queue_used = self.queue_used.write().unwrap();
            queue_used.remove(&seed).unwrap()
        };

        if let Some(russians) = self.russians.as_ref() {
            query
                .as_raw_slice()
                .iter()
                .enumerate()
                .map(|(i, q)| russians[i][*q as usize].clone())
                .for_each(|x| answer ^= x);
        } else {
            query
                .iter()
                .zip(self.db.iter())
                .filter(|(q, _)| **q)
                .for_each(|(_, x)| answer ^= x.clone());
        }

        answer
    }
}
