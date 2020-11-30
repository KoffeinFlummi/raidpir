//! Methods for making RAID-PIR queries and combining the responses.

use std::ops::BitXorAssign;

use bitvec::prelude::*;

use crate::util::*;

/// RaidPir client.
#[derive(Debug)]
pub struct RaidPirClient {
    blocks: usize,
    blocks_padded: usize,
    servers: usize,
    redundancy: usize
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
    pub fn new(
        blocks: usize,
        servers: usize,
        redundancy: usize,
    ) -> Self {
        // TODO: move to param type?

        // blocks per server has to be a multiple of the size of usize to make
        // the math easier/faster. Since we don't know whether the server is 32
        // or 64 bit, assume 64 bit.
        let blocks_padded = if blocks % (servers * 64) == 0 {
            blocks
        } else {
            blocks + servers * 64 - (blocks % (servers * 64))
        };

        assert!(blocks_padded % servers == 0);
        assert!((blocks_padded / servers) % std::mem::size_of::<&usize>() == 0);
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
     * assert_eq!(queries[0].len(), 64);
     * ```
     */
    pub fn query(&self, index: usize, seeds: &Vec<u64>) -> Vec<BitVec> {
        assert!(index < self.blocks);
        assert!(seeds.len() == self.servers);

        let mut query: BitVec<Lsb0> = BitVec::new();
        query.resize(self.blocks_padded, false);
        query.set(index, true);

        let blocks_per_server = self.blocks_padded / self.servers;

        for i in 0..self.servers {
            let random_bits = rand_bitvec(seeds[i], blocks_per_server * (self.redundancy - 1));

            // BitSlice's as_raw_slice methods only cover the completely covered
            // elements, so if in the future blocks_per_server is not a multiple
            // of the size of usize, this would have to be changed.
            let query_slice = query.as_raw_slice_mut();

            // TODO: multithreading, SIMD
            for (j, r) in random_bits.as_raw_slice().iter().enumerate() {
                // index of first non-flip chunk of query for this server,
                // divided by BitVec element size.
                let first_index = (i+1) * blocks_per_server / (std::mem::size_of::<&usize>() * 8);
                query_slice[(first_index + j) % query_slice.len()] ^= r;
            }
        }

        // split query into server chunks
        query
            .into_vec()
            .chunks(blocks_per_server / (std::mem::size_of::<&usize>() * 8))
            .map(|x| BitVec::from_vec(x.to_vec()))
            .collect()
    }

    /**
     * Combine responses from servers to calculate queried element.
     */
    pub fn combine<T: Sized + Default + BitXorAssign>(&self, responses: Vec<T>) -> T {
        assert!(responses.len() == self.servers);

        let mut data = T::default();

        for r in responses {
            data ^= r;
        }

        data
    }
}