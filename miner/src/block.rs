use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub const DIFFICULTY: u32 = 25;

#[derive(Default)]
pub struct BlockHasher {
    id: u64,
}

impl std::hash::Hasher for BlockHasher {
    fn finish(&self) -> u64 {
        self.id
    }

    fn write_u64(&mut self, id: u64) {
        self.id = id;
    }

    fn write(&mut self, _: &[u8]) {
        unimplemented!()
    }
}

pub type BlockIdHasher = std::hash::BuildHasherDefault<BlockHasher>;
pub type BlockHashSet = HashSet<u64, BlockIdHasher>;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Block {
    pub parent_hash: Vec<u8>,
    pub miner: String,
    pub dancemove: DanceMove,
    pub nonce: u64,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum DanceMove {
    #[default]
    Y = 1,
    M = 2,
    C = 3,
    A = 4,
}

impl Block {
    pub fn new(parent_hash: Vec<u8>, miner: String, nonce: u64, dancemove: DanceMove) -> Self {
        Block {
            parent_hash,
            miner,
            dancemove,
            nonce,
        }
    }

    pub fn hash_block(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();

        hasher.update(&self.parent_hash);
        hasher.update(self.miner.as_bytes());
        hasher.update((self.dancemove as u8).to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());

        let result = hasher.finalize();

        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }

   
    pub fn solve_block<R: RngCore>(
        &mut self,
        rng: &mut R,
        difficulty: u32,
        max_iteration: Option<u64>,
    ) -> Option<Vec<u8>> {
        let max = max_iteration.unwrap_or(u64::MAX);
        let mut i = 0u64;

        while i < max {
            let hash = self.hash_block();

            if Block::pow_check(&hash, difficulty) {
                return Some(hash.to_vec());
            }

            self.nonce = rng.next_u64();
            i += 1;
        }

        None
    }

    pub fn pow_check(hash: &[u8], difficulty: u32) -> bool {
    let mut remaining = difficulty as usize;
    for &byte in hash {
        if remaining == 0 {
            return true;
        }
        if remaining >= 8 {
            if byte != 0 { return false; }
            remaining -= 8;
        } else {
            // les `remaining` bits de poids fort doivent être 0
            return (byte >> (8 - remaining)) == 0;
        }
    }
    remaining == 0  
}



    pub fn is_block_valid(&self, difficulty: u32) -> Result<(), &'static str> {
        if self.miner == "changemeyoufool" {
            return Err("miner name is changemeyoufool");
        }

        if self.miner == "Genesis" {
            return Err("miner name is Genesis");
        }

        if !Block::pow_check(&self.hash_block(), difficulty) {
            return Err("invalid proof of work");
        }

        Ok(())
    }

    
    pub fn is_genesis(&self, difficulty: u32) -> bool {
        if self.miner != "Genesis" {
            return false;
        }

        if !self.parent_hash.is_empty() {
            return false;
        }

        Block::pow_check(&self.hash_block(), difficulty)

    }
}


impl crate::simpletree::Parenting for Block {
    fn is_parent(&self, parent_id: &[u8]) -> bool {
        self.hash_block().as_slice() == parent_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::Rng;
    use rand::SeedableRng;

    #[test]
    fn test_pow_check() {
        let hash_with_zeros = vec![0x00, 0x00, 0x00, 0xFF];
        assert!(Block::pow_check(&hash_with_zeros, 24));

        let hash_without_zeros = vec![0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!Block::pow_check(&hash_without_zeros, 1));

        assert!(Block::pow_check(&hash_without_zeros, 0));
    }

    #[test]
    fn test_solve_block() {
        let mut block = Block {
            parent_hash: vec![],
            miner: "test".to_string(),
            nonce: 0,
            dancemove: DanceMove::Y,
        };

        let mut rng = StdRng::seed_from_u64(42);
        block.nonce = rng.random();

        for difficulty in 5..10 {
            if difficulty % 2 == 0 {
                block.dancemove = DanceMove::A;
            } else {
                block.dancemove = DanceMove::M;
            }

            let hash = block.solve_block(&mut rng, difficulty, None).unwrap();

            assert!(Block::pow_check(&hash, difficulty));
            assert_ne!(block.nonce, 0);
        }
    }

    #[test]
    fn test_new_genesis() {
        let mut genesis =
            Block::new(Vec::new(), "Genesis".to_string(), 42, DanceMove::C);

        let mut rng = StdRng::seed_from_u64(42);
        genesis.nonce = rng.random();

        genesis.solve_block(&mut rng, 10, None).unwrap();

        assert!(genesis.is_genesis(10));
    }
}