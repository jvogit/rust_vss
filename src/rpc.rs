use std::sync::mpsc::Sender;

use num_bigint::BigUint;

/// (i, P(i))
pub type Share = (BigUint, BigUint);

/// Share, g, c, p, q, t
pub type ShareInfo = (Share, BigUint, Vec<BigUint>, BigUint, BigUint, usize);

#[derive(Debug, Clone)]
pub enum RPC {
    Ping(usize),
    RegSender(usize, Sender<RPC>),
    RegShare(ShareInfo),
    ReconstructShare(usize, Share),
    Reconstruct(Sender<usize>),
}
