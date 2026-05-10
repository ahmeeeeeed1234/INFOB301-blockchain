use block::Block;
use block::DanceMove;
use block::DIFFICULTY;
use clap::{Parser, Subcommand};
use network::NetworkConnector;
use rand::Rng;
use simpletree::TreeNode;
use std::fmt;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const MY_NAME: &str = "AhmedMiner";

#[derive(Default, Debug)]
struct Blockchain {
    blocks: TreeNode<Block>,
}

impl Blockchain {
    pub fn new_from_genesis(genesis: Block) -> Self {
        Blockchain {
            blocks: TreeNode::new(genesis),
        }
    }

    pub fn new_from_genesis_and_vec(
        genesis: Block,
        blocks: Vec<Block>,
    ) -> (Self, Vec<Block>) {
        let mut chain = Blockchain::new_from_genesis(genesis);
        let mut remaining = blocks;

        loop {
            let mut placed_one = false;
            let mut still_remaining = Vec::new();

            for b in remaining.into_iter() {
                let parent_hash = b.parent_hash.clone();

                if let Some(parent_node) = chain.blocks.look_for_parent(&parent_hash) {
                    let already_there = parent_node
                        .children()
                        .iter()
                        .any(|c| c.value() == &b);

                    if !already_there {
                        parent_node.insert(b);
                        placed_one = true;
                    }
                } else {
                    still_remaining.push(b);
                }
            }

            remaining = still_remaining;

            if !placed_one {
                break;
            }
        }

        (chain, remaining)
    }

    fn print_tree(
        &self,
        f: &mut fmt::Formatter<'_>,
        node: &TreeNode<Block>,
        prefixes: &mut Vec<bool>,
    ) -> fmt::Result {
        if !prefixes.is_empty() {
            for &is_last in &prefixes[..prefixes.len() - 1] {
                write!(f, "{}", if is_last { "    " } else { "│   " })?;
            }

            let is_last = *prefixes.last().unwrap();
            write!(f, "{}", if is_last { "└── " } else { "├── " })?;
        }

        let block = node.value();
        writeln!(f, "{} (nonce: {})", block.miner, block.nonce)?;

        let child_count = node.children().len();

        for (i, child) in node.children().iter().enumerate() {
            prefixes.push(i == child_count - 1);
            self.print_tree(f, child, prefixes)?;
            prefixes.pop();
        }

        Ok(())
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print_tree(f, &self.blocks, &mut Vec::new())
    }
}

fn longest_chain(node: &TreeNode<Block>) -> Vec<Block> {
    let mut best: Vec<Block> = Vec::new();

    for child in node.children() {
        let sub = longest_chain(child);

        if sub.len() > best.len() {
            best = sub;
        }
    }

    let mut result = vec![node.value().clone()];
    result.extend(best);
    result
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    action: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Mine {
        #[arg(short, default_value_t = DIFFICULTY)]
        difficulty: u32,

        #[arg(short, default_value_t = String::from(MY_NAME))]
        miner_name: String,

        #[arg(long)]
        max_iter: Option<u64>,
    },

    Print {
        #[arg(short, default_value_t = DIFFICULTY)]
        difficulty: u32,
    },
}

fn find_genesis(blocks: &[Block], difficulty: u32) -> Option<Block> {
    for b in blocks {
        if b.is_genesis(difficulty) {
            return Some(b.clone());
        }
    }

    None
}

fn create_genesis(difficulty: u32) -> Block {
    let mut rng = rand::rng();

    let mut g = Block::new(
        Vec::new(),
        "Genesis".to_string(),
        rng.random(),
        DanceMove::Y,
    );

    g.solve_block(&mut rng, difficulty, None);
    g
}

fn mine(difficulty: u32, miner_name: String, max_iter: Option<u64>) {
    let (tx1, rx1) = mpsc::sync_channel(1);
    let (tx2, rx2) = mpsc::channel();

    thread::spawn(move || {
        let mut net = NetworkConnector::new(tx1, rx2);
        net.sync().expect("Network failure");
    });

    let mut rng = rand::rng();

    let mut genesis: Option<Block> = None;

    while genesis.is_none() {
        if let Ok(blocks) = rx1.recv() {
            genesis = find_genesis(&blocks, difficulty);

            if genesis.is_none() {
                let g = create_genesis(difficulty);
                println!("Pas de genesis trouve, on envoie le notre");

                tx2.send(g.clone()).ok();
                genesis = Some(g);
            }
        }
    }

    let genesis = genesis.unwrap();
    println!("Genesis trouve: nonce={}", genesis.nonce);

    loop {
        let mut latest_blocks: Vec<Block> = Vec::new();

        match rx1.try_recv() {
            Ok(blocks) => latest_blocks = blocks,
            Err(_) => {}
        }

        let blocks_without_genesis: Vec<Block> = latest_blocks
            .into_iter()
            .filter(|b| b != &genesis)
            .collect();

        let (chain, _orphans) =
            Blockchain::new_from_genesis_and_vec(genesis.clone(), blocks_without_genesis);

        let longest = longest_chain(&chain.blocks);
        let parent = longest.last().unwrap().clone();
        let parent_hash = parent.hash_block().to_vec();

        let dance = match rng.random_range(0..4) {
            0 => DanceMove::Y,
            1 => DanceMove::M,
            2 => DanceMove::C,
            _ => DanceMove::A,
        };

        let mut new_block = Block::new(
            parent_hash,
            miner_name.clone(),
            rng.random(),
            dance,
        );

        println!("Mining sur la chaine de longueur {}...", longest.len());

        if let Some(_) = new_block.solve_block(&mut rng, difficulty, max_iter) {
            println!(
                "Bloc trouve ! (nonce={}, dance={:?})",
                new_block.nonce, new_block.dancemove
            );

            tx2.send(new_block).ok();
        } else {
            println!("Pas de bloc trouve dans la limite, on recommence");
        }

        thread::sleep(Duration::from_millis(500));
    }
}

fn print_chain(difficulty: u32) {
    let blocks = match network::get_blocks() {
        Ok(b) => b,
        Err(e) => {
            println!("Erreur reseau: {:?}", e);
            return;
        }
    };

    let genesis = match find_genesis(&blocks, difficulty) {
        Some(g) => g,
        None => {
            println!("Pas de bloc genesis trouve sur le serveur");
            return;
        }
    };

    let blocks_without_genesis: Vec<Block> =
        blocks.into_iter().filter(|b| b != &genesis).collect();

    let (chain, orphans) =
        Blockchain::new_from_genesis_and_vec(genesis, blocks_without_genesis);

    println!("{}", chain);

    if !orphans.is_empty() {
        println!("\n{} bloc(s) orphelin(s) ignore(s)", orphans.len());
    }

    let longest = longest_chain(&chain.blocks);
    println!("\nPlus longue chaine: {} blocs", longest.len());
}

fn main() {
    let args = Args::parse();

    match &args.action {
        Some(Commands::Mine {
            difficulty,
            miner_name,
            max_iter,
        }) => {
            mine(*difficulty, miner_name.clone(), *max_iter);
        }

        Some(Commands::Print { difficulty }) => {
            print_chain(*difficulty);
        }

        None => {
            println!("Use --help to see available commands");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block(parent_hash: &[u8], nonce_init: u64, miner: &str) -> Block {
        Block::new(
            parent_hash.to_vec(),
            miner.to_string(),
            nonce_init,
            DanceMove::Y,
        )
    }

    #[test]
    fn test_empty_blocks() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let (blockchain, _) =
            Blockchain::new_from_genesis_and_vec(genesis, vec![]);

        assert_eq!(blockchain.blocks.children().len(), 0);
    }

    #[test]
    fn test_single_valid_block() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis_hash = genesis.hash_block().to_vec();

        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let (blockchain, _) =
            Blockchain::new_from_genesis_and_vec(genesis, vec![block1]);

        let root = &blockchain.blocks;

        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].value().miner, "miner1");
    }

    #[test]
    fn test_multiple_levels() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis_hash = genesis.hash_block().to_vec();

        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let block1_hash = block1.hash_block().to_vec();

        let block2 = create_test_block(&genesis_hash, 43, "miner2");
        let block3 = create_test_block(&block1_hash, 44, "miner3");

        let (blockchain, remaining) =
            Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![block1, block2, block3],
            );

        let root = &blockchain.blocks;

        assert_eq!(root.children().len(), 2);

        let block1_node = root
            .children()
            .iter()
            .find(|n| n.value().miner == "miner1")
            .unwrap();

        assert_eq!(block1_node.children().len(), 1);
        assert_eq!(block1_node.children()[0].value().miner, "miner3");
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_orphaned_blocks() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let fake_hash = vec![0xFF; 32];

        let valid_block =
            create_test_block(&genesis.hash_block().to_vec(), 42, "miner1");
        let orphan_block = create_test_block(&fake_hash, 10, "miner2");

        let (blockchain, _) =
            Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![valid_block, orphan_block],
            );

        assert_eq!(blockchain.blocks.children().len(), 1);
        assert_eq!(blockchain.blocks.children()[0].value().miner, "miner1");
    }

    #[test]
    fn test_duplicate_valid_blocks() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis_hash = genesis.hash_block().to_vec();

        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let block1_hash = block1.hash_block().to_vec();

        let block2 = create_test_block(&genesis_hash, 43, "miner2");
        let block3 = create_test_block(&block1_hash, 43, "miner3");

        let (blockchain, _) =
            Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![block1, block2, block3],
            );

        let root = &blockchain.blocks;

        assert_eq!(root.children().len(), 2);

        let block1_node = root
            .children()
            .iter()
            .find(|n| n.value().miner == "miner1")
            .unwrap();

        assert_eq!(block1_node.children().len(), 0);
    }

    #[test]
    fn test_complex_structure() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis_hash = genesis.hash_block().to_vec();

        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let block1_hash = block1.hash_block().to_vec();

        let block2 = create_test_block(&genesis_hash, 43, "miner2");
        let block2_hash = block2.hash_block().to_vec();

        let block3 = create_test_block(&block1_hash, 44, "miner3");
        let block4 = create_test_block(&block2_hash, 45, "miner4");
        let block5 = create_test_block(&block2_hash, 46, "miner5");

        let (blockchain, _) =
            Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![block1, block2, block3, block4, block5],
            );

        let root = &blockchain.blocks;

        assert_eq!(root.children().len(), 2);

        let block1_node = root
            .children()
            .iter()
            .find(|n| n.value().miner == "miner1")
            .unwrap();

        assert_eq!(block1_node.children().len(), 1);
        assert_eq!(block1_node.children()[0].value().miner, "miner3");

        let block2_node = root
            .children()
            .iter()
            .find(|n| n.value().miner == "miner2")
            .unwrap();

        assert_eq!(block2_node.children().len(), 2);

        assert!(block2_node
            .children()
            .iter()
            .any(|n| n.value().miner == "miner4"));

        assert!(block2_node
            .children()
            .iter()
            .any(|n| n.value().miner == "miner5"));
    }

    #[test]
    fn test_multiple_genesis() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis2 = create_test_block(&[], 42, "Genesis");

        let genesis_hash = genesis.hash_block().to_vec();
        let genesis2_hash = genesis2.hash_block().to_vec();

        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let block1_hash = block1.hash_block().to_vec();

        let block2 = create_test_block(&genesis_hash, 43, "miner2");
        let block3 = create_test_block(&block1_hash, 44, "miner3");

        let block4 = create_test_block(&genesis2_hash, 42, "miner1");

        let (_, remaining) =
            Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![block1, block2, block3, block4],
            );

        assert_eq!(remaining.len(), 1);
    }
}

mod block;
mod network;
mod simpletree;