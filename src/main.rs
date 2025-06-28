use core::panic;

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

impl PseudoTcpClient {
    fn new(port: u16) -> Self {
        Self {
            port,
            state: TcpState::Closed,
            seq: 1000,
            ack: 0,
            peer_seq: 0,
        }
    }

    fn send_syn(&mut self, to: u16) -> TcpSegment {
        self.state = TcpState::SynSent;
        TcpSegment::syn(self.port, to, self.seq)
    }

    fn receive(&mut self, seg: &TcpSegment) -> Option<TcpSegment> {
        match self.state {
            TcpState::Closed => {
                panic!()
            }
            TcpState::SynSent => {
                panic!()
            }
            TcpState::SynReceived => {
                panic!()
            }
            TcpState::Established => {
                panic!()
            }
        }
    }

    fn send_data(&mut self, to: u16, msg: &str) -> TcpSegment {
        self.seq += msg.len() as u32;
        TcpSegment::data(self.port, to, self.seq, self.ack, msg.as_bytes().to_vec())
    }
}

fn main() {
    println!("Hello, world!");
}
