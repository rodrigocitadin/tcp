#[derive(Debug)]
enum TcpState {
    Closed,
    SynSent,
    SynReceived,
    Established,
}

#[derive(Debug, Clone)]
struct TcpSegment {
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    syn: bool,
    ack_flag: bool,
    payload: Vec<u8>,
}

struct PseudoTcpClient {
    port: u16,
    state: TcpState,
    seq: u32,
    ack: u32,
    peer_seq: u32,
}

fn main() {
    println!("Hello, world!");
}

