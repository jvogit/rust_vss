use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

use num::ToPrimitive;

use crate::rpc::{Share, ShareInfo, RPC};
use crate::vss;

pub struct Player {
    id: usize,
    rx: Receiver<RPC>,
    senders: HashMap<usize, Sender<RPC>>,
    share_info: Option<ShareInfo>,
}

impl Player {
    pub fn new(id: usize) -> (Sender<RPC>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<RPC>();
        let mut player = Player {
            id,
            rx,
            senders: HashMap::new(),
            share_info: None,
        };
        let handler = thread::spawn(move || {
            player.start();
        });

        (tx, handler)
    }

    fn start(&mut self) {
        let mut reconstruct_send: Option<Sender<usize>> = None;
        let mut senders_shares: HashMap<usize, Share> = HashMap::new();
        
        while let Ok(rpc) = self.rx.recv() {
            match rpc {
                RPC::Ping(other_id) => println!("{} Pong to {}", self.id, other_id),
                RPC::RegSender(other_id, sender) => {
                    println!("{} RegSender {}", self.id, other_id);
                    self.senders.insert(other_id, sender);
                }
                RPC::RegShare(share_info) => {
                    println!("{} RegShare", self.id);
                    let (share, g, c, p, _, _) = &share_info;
                    let is_verified = vss::verify_share(&share.0, &share.1, g, c, p);
                    if !is_verified {
                        println!("{} received an invalid share", self.id);
                        return;
                    }

                    self.share_info = Some(share_info);
                }
                RPC::ReconstructShare(other_id, other_share) => {
                    println!("{} ReconstructShare {}", self.id, other_id);
                    if let Some((_, g, c, p, q, t)) = &self.share_info {
                        let is_verified =
                            vss::verify_share(&other_share.0, &other_share.1, g, c, p);

                        if !is_verified {
                            println!("{} received an invalid share for {}", self.id, other_id);
                            return;
                        }

                        senders_shares.insert(other_id, other_share);

                        if senders_shares.len() >= *t {
                            let shares = senders_shares
                                .values()
                                .into_iter()
                                .map(|x| x.clone())
                                .collect();
                            let reconstruct_secret = vss::reconstruct(&shares, q);

                            if let Some(s) = reconstruct_send.take() {
                                s.send(reconstruct_secret.to_usize().unwrap());
                            }

                            senders_shares.clear();
                        }
                    }
                }
                RPC::Reconstruct(s) => {
                    println!("{} Reconstruct", self.id);
                    if let Some((share, _, _, _, _, _)) = &self.share_info {
                        reconstruct_send = Some(s);
                        self.broadcast(RPC::ReconstructShare(self.id, share.clone()));
                    }
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
