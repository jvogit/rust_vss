use std::{sync::mpsc::Sender, thread::JoinHandle};

use rust_vss::{dealer::Dealer, player::Player, rpc::RPC};

fn main() {
    let dealer = Dealer::new(5, 4, 1234);
    let n = 3;
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

    // Wait on all players
    registered.into_iter().for_each(|(_, _, handle)| {
        handle.join();
    });
}
