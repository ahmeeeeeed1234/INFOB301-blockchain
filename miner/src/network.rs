use crate::Block;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread::sleep;
use std::time::Duration;

const URL: &str = "http://localhost:8080";
const REQUEST_PAUSE_IN_SECONDS: u64 = 1;

pub struct NetworkConnector {
    tx: SyncSender<Vec<Block>>,
    rx: Receiver<Block>,
}

impl NetworkConnector {
    pub fn new(tx: SyncSender<Vec<Block>>, rx: Receiver<Block>) -> Self {
        NetworkConnector { tx, rx }
    }

    pub fn sync(&mut self) -> reqwest::Result<()> {
        loop {
            let blocks = get_blocks()?;
            self.tx.send(blocks).ok();

            while let Ok(block) = self.rx.try_recv() {
                send_block(&block)?;
            }

            sleep(Duration::from_secs(REQUEST_PAUSE_IN_SECONDS));
        }
    }
}

pub fn get_blocks() -> reqwest::Result<Vec<Block>> {
    let fullurl = format!("{}/blocks", URL);

    let blocks: Vec<Block> = reqwest::blocking::get(fullurl)?.json()?;

    Ok(blocks)
}

pub fn send_block(block: &Block) -> reqwest::Result<()> {
    let fullurl = format!("{}/postblock", URL);

    let client = reqwest::blocking::Client::new();
    let response = client.post(fullurl).json(block).send()?;

    if response.status().is_client_error() {
        println!("Erreur serveur: {:?}", response.text()?);
    }

    Ok(())
}

pub fn simple_sync_loop() -> reqwest::Result<()> {
    loop {
        let blocks = get_blocks()?;

        println!("{} blocs recus depuis le serveur", blocks.len());

        sleep(Duration::from_secs(REQUEST_PAUSE_IN_SECONDS));
    }
}