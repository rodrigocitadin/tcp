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

impl TcpSegment {
    fn syn(src: u16, dst: u16, seq: u32) -> Self {
        Self {
            src_port: src,
            dst_port: dst,
            seq,
            ack: 0,
            syn: true,
            ack_flag: false,
            payload: vec![],
        }
    }

    fn syn_ack(src: u16, dst: u16, seq: u32, ack: u32) -> Self {
        Self {
            src_port: src,
            dst_port: dst,
            seq,
            ack,
            syn: true,
            ack_flag: true,
            payload: vec![],
        }
    }

    fn ack(src: u16, dst: u16, seq: u32, ack: u32) -> Self {
        Self {
            src_port: src,
            dst_port: dst,
            seq,
            ack,
            syn: false,
            ack_flag: true,
            payload: vec![],
        }
    }

    fn data(src: u16, dst: u16, seq: u32, ack: u32, data: Vec<u8>) -> Self {
        Self {
            src_port: src,
            dst_port: dst,
            seq,
            ack,
            syn: false,
            ack_flag: true,
            payload: data,
        }
    }
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

