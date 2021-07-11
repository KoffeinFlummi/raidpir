//! Methods for making RAID-PIR queries and combining the responses.

use std::ops::BitXorAssign;

use bitvec::prelude::*;
use rayon::prelude::*;

use crate::util::*;

/// RaidPir client.
#[derive(Debug)]
pub struct RaidPirClient {
    blocks: usize,
    blocks_padded: usize,
    servers: usize,
    redundancy: usize,
}

impl RaidPirClient {
    /**
     * Create a new client object.
     *
     * ```
     * use raidpir::client::RaidPirClient;
     *
     * let client = RaidPirClient::new(12, 4, 3);
     * ```
     */
    pub fn new(blocks: usize, servers: usize, redundancy: usize) -> Self {
        // TODO: move to param type?

        // blocks per server has to be a multiple of the size of usize to make
        // the math easier/faster. Since we don't know whether the server is 32
        // or 64 bit, assume 64 bit.
        let blocks_padded = if blocks % (servers * 8) == 0 {
            blocks
        } else {
            blocks + servers * 8 - (blocks % (servers * 8))
        };

        assert!(blocks_padded % servers == 0);
        assert!((blocks_padded / servers) % 8 == 0);
        assert!(redundancy >= 2 && redundancy <= servers);

        Self {
            blocks,
            blocks_padded,
            servers,
            redundancy,
        }
    }

    /**
     * Calculate query for the given index and seeds.
     *
     * ```
     * use raidpir::client::RaidPirClient;
     *
     * let client = RaidPirClient::new(12, 4, 3);
     * let queries = client.query(3, &vec![0, 12, 4, 8]);
     *
     * assert_eq!(queries.len(), 4);
     * assert_eq!(queries[0].len(), 8);
     * ```
     */
    pub fn query(&self, index: usize, seeds: &Vec<u128>) -> Vec<BitVec::<Lsb0,u8>> {
        assert!(index < self.blocks);
        assert!(seeds.len() == self.servers);

        let mut query: BitVec<Lsb0,u8> = BitVec::new();
        query.resize(self.blocks_padded, false);
        query.set(index, true);

        let blocks_per_server = self.blocks_padded / self.servers;

        let random_bits: Vec<BitVec<Lsb0,u8>> = seeds
            .par_iter()
            .map(|s| rand_bitvec(*s, blocks_per_server * (self.redundancy - 1)))
            .collect();

        // BitSlice's as_raw_slice methods only cover the completely covered
        // elements, so if in the future blocks_per_server is not a multiple
        // of the size of usize, this would have to be changed.
        let query_slice = query.as_raw_slice_mut();

        for random in random_bits.iter() {
            // Rotate left one chunk so we start at blocks_per_server * (i+1).
            // This way we can simply use iter for performing the XORing, which
            // makes using SIMD and parallelizing later easier. By the time
            // we're done with the loop, the query is back where it started.
            //
            // Counterintuitively, this is actually faster (even without SIMD
            // and multithreading) than a for loop with index % len.
            query_slice.rotate_left(blocks_per_server / 8);

            xor_into_slice(query_slice, random.as_raw_slice());
        }

        // split query into server chunks
        query
            .into_vec()
            .chunks(blocks_per_server / 8)
            .map(|x| BitVec::from_vec(x.to_vec()))
            .collect()
    }

    /**
     * Combine responses from servers to calculate queried element.
     */
    pub fn combine<T: Clone + Default + BitXorAssign>(&self, responses: Vec<T>) -> T {
        assert!(responses.len() == self.servers);

        let mut data = responses[0].clone();
        for i in 1..responses.len() {
            data ^= responses[i].clone();
        }

        data
    }
}
