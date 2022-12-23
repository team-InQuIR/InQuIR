pub enum RemoteOp {
    LocalCX,
    RCX,
    RSwap,
    Move(String, u32, u32),
}

pub trait RemoteOpRouter {
    fn current_pos(&self, id: &String) -> u32;
    fn next(&mut self, id1: &String, id2: &String) -> RemoteOp;
}
