use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::rpc::RPC;

pub struct Player {
    id: usize,
    rx: Receiver<RPC>,
    senders: HashMap<usize, Sender<RPC>>,
}

impl Player {
    pub fn new(id: usize) -> (Sender<RPC>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<RPC>();
        let mut player = Player {
            id,
            rx,
            senders: HashMap::new(),
        };

        let handler = thread::spawn(move || {
            player.start();
        });

        (tx, handler)
    }

    fn start(&mut self) {
        while let Ok(rpc) = self.rx.recv() {
            match rpc {
                RPC::Ping(id) => println!("{} Pong to {}", self.id, id),
                RPC::RegSender(id, sender) => {
                    println!("{} RegSender {}", self.id, id);
                    self.senders.insert(id, sender);
                }
            }
        }
    }

    fn broadcast(&self, rpc: RPC) {
        self.senders.iter().for_each(|(to, s)| {
            if let Err(res) = s.send(rpc.clone()) {
                println!("{} error while broadcasting {}: {}", self.id, to, res);
            }
        });
    }
}
