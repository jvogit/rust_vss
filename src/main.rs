use std::{
    sync::mpsc::{self, Sender},
    thread::JoinHandle,
};

use rust_vss::{dealer::Dealer, player::Player, rpc::RPC};

fn main() {
    let n = 5;
    let dealer = Dealer::new(n, 3, 1234);
    let mut registered: Vec<(usize, Sender<RPC>, JoinHandle<()>)> = vec![];

    for id in 1..=n {
        let (sender, handler) = Player::new(id);

        registered.iter().for_each(|(other_id, other_sender, _)| {
            let reg_sender = RPC::RegSender(id, sender.clone());
            other_sender.send(reg_sender);
            let reg_other = RPC::RegSender(other_id.clone(), other_sender.clone());
            sender.send(reg_other);
        });
        registered.push((id, sender, handler));
    }
    dealer.propagate(&registered.iter().map(|(_, s, _)| s.clone()).collect());

    let (sender, receiver) = mpsc::channel();

    registered.iter().for_each(|(_, s, _)| {
        s.send(RPC::Reconstruct(sender.clone()));
    });

    match receiver.recv() {
        Ok(secret) => println!("Reconstructed secret! {}", secret),
        Err(err) => println!("An error occured while reconstructing secret {}", err),
    }

    // Wait on all players
    registered.into_iter().for_each(|(_, _, handle)| {
        handle.join();
    });
}
