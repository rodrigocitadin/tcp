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

struct TcpClient {
    port: u16,
    state: TcpState,
    seq: u32,
    ack: u32,
    peer_seq: u32,
}

impl TcpClient {
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
                if seg.syn && !seg.ack_flag {
                    // Received SYN → respond with SYN-ACK
                    self.state = TcpState::SynReceived;
                    self.peer_seq = seg.seq;
                    self.ack = seg.seq + 1;
                    Some(TcpSegment::syn_ack(
                        self.port,
                        seg.src_port,
                        self.seq,
                        self.ack,
                    ))
                } else {
                    None
                }
            }
            TcpState::SynSent => {
                if seg.syn && seg.ack_flag {
                    // Received SYN-ACK → send ACK
                    self.peer_seq = seg.seq;
                    self.ack = seg.seq + 1;
                    self.state = TcpState::Established;
                    Some(TcpSegment::ack(
                        self.port,
                        seg.src_port,
                        self.seq + 1,
                        self.ack,
                    ))
                } else {
                    None
                }
            }
            TcpState::SynReceived => {
                if seg.ack_flag && !seg.syn {
                    // Final ACK in 3-way handshake
                    self.state = TcpState::Established;
                    None
                } else {
                    None
                }
            }
            TcpState::Established => {
                if !seg.payload.is_empty() {
                    println!(
                        "Client {} received data: {:?}",
                        self.port,
                        String::from_utf8_lossy(&seg.payload)
                    );
                    None
                } else {
                    None
                }
            }
        }
    }

    fn send_data(&mut self, to: u16, msg: &str) -> TcpSegment {
        self.seq += msg.len() as u32;
        TcpSegment::data(self.port, to, self.seq, self.ack, msg.as_bytes().to_vec())
    }
}

fn main() {
    let mut client_a = TcpClient::new(1000);
    let mut client_b = TcpClient::new(2000);

    // Step 1: A sends SYN to B
    let syn = client_a.send_syn(client_b.port);
    println!("[A -> B] SYN: {:?}", syn);

    // Step 2: B receives SYN and responds with SYN-ACK
    let syn_ack = client_b.receive(&syn).unwrap();
    println!("[B -> A] SYN-ACK: {:?}", syn_ack);

    // Step 3: A receives SYN-ACK and responds with ACK
    let ack = client_a.receive(&syn_ack).unwrap();
    println!("[A -> B] ACK: {:?}", ack);

    // Step 4: B receives ACK
    client_b.receive(&ack);

    // Now both are Established
    assert!(matches!(client_a.state, TcpState::Established));
    assert!(matches!(client_b.state, TcpState::Established));

    println!("Connection established!\n");

    // Exchange data
    let msg = client_a.send_data(client_b.port, "hello from A");
    println!("[A -> B] Data: {:?}", msg);
    client_b.receive(&msg);

    let reply = client_b.send_data(client_a.port, "hi from B");
    println!("[B -> A] Data: {:?}", reply);
    client_a.receive(&reply);
}
