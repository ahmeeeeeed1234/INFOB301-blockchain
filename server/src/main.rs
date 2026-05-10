#[macro_use]
extern crate rouille;

use clap::Parser;
use miner::block::Block;
use miner::block::BlockIdHasher;
use miner::block::DIFFICULTY;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short)]
    address: Option<String>,

    #[arg(short, default_value_t = 8080)]
    port: u16,

    #[arg(short, default_value_t = DIFFICULTY)]
    difficulty: u32,
}

type BlockHashMap<V> = HashMap<u64, V, BlockIdHasher>;

fn main() {
    let args = Args::parse();
    let address = args.address.unwrap_or("0.0.0.0".to_string());

    let db = Mutex::new(BlockHashMap::<Block>::default());

    println!("Now listening on {:?}:{:?}", address, args.port);

    rouille::start_server(format!("{}:{}", address, args.port), move |request| {
        rouille::log(request, std::io::stdout(), || {
            router!(request,
                (GET) (/blocks) => {
                    let db = db.lock().unwrap();

                    rouille::Response::json(
                        &db.values().cloned().collect::<Vec<Block>>()
                    )
                },

                (POST) (/postblock) => {
                    if request.header("Content-Type") != Some("application/json") {
                        return rouille::Response::text(
                            "Expected Content-Type: application/json"
                        )
                        .with_status_code(400);
                    }

                    let block: Block = match rouille::input::json_input(request) {
                        Ok(block) => block,

                        Err(e) => {
                            eprintln!("JSON parse error: {:?}", e);

                            return rouille::Response::text("Invalid JSON format")
                                .with_status_code(400);
                        }
                    };

                    let mut db = db.lock().unwrap();

                    if db.contains_key(&block.nonce) {
                        return rouille::Response::text("Block already exists")
                            .with_status_code(400);
                    }

                    if !Block::pow_check(&block.hash_block(), args.difficulty) {
                        return rouille::Response::text("Invalid proof-of-work")
                            .with_status_code(400);
                    }

                    db.insert(block.nonce, block);

                    rouille::Response::text("Block accepted")
                        .with_status_code(200)
                },

                _ => rouille::Response::empty_404()
            )
        })
    });
}