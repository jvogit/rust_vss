use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum RPC {
    Ping(usize),
    RegSender(usize, Sender<RPC>),
}
