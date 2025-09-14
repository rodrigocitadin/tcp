use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

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

    fn to_bytes(&self) -> [u8; 20] {
        let mut buf = [0u8; 20];
        buf[0..2].copy_from_slice(&self.src_port.to_be_bytes());
        buf[2..4].copy_from_slice(&self.dst_port.to_be_bytes());
        buf[4..8].copy_from_slice(&self.seq.to_be_bytes());
        buf[8..12].copy_from_slice(&self.ack.to_be_bytes());
        buf[12] = 0x50;
        buf[13] = (if self.syn { 0x02 } else { 0x00 }) | (if self.ack_flag { 0x10 } else { 0x00 });
        buf[14..16].copy_from_slice(&64240u16.to_be_bytes());
        buf[16..18].copy_from_slice(&0u16.to_be_bytes());
        buf[18..20].copy_from_slice(&0u16.to_be_bytes());
        buf
    }

    fn with_checksum(mut self, src_ip: [u8; 4], dst_ip: [u8; 4]) -> Self {
        let checksum = compute_tcp_checksum(&self, src_ip, dst_ip);
        self.payload.insert(0, (checksum >> 8) as u8);
        self.payload.insert(1, (checksum & 0xFF) as u8);
        self
    }
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut chunks = data.chunks_exact(2);
    for chunk in &mut chunks {
        let word = u16::from_be_bytes([chunk[0], chunk[1]]);
        sum = sum.wrapping_add(word as u32);
    }
    if let Some(&b) = chunks.remainder().first() {
        sum = sum.wrapping_add((b as u32) << 8);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !sum as u16
}

fn compute_tcp_checksum(segment: &TcpSegment, src_ip: [u8; 4], dst_ip: [u8; 4]) -> u16 {
    let tcp_len = 20 + segment.payload.len();
    let mut pseudo = Vec::with_capacity(12 + tcp_len);
    pseudo.extend_from_slice(&src_ip);
    pseudo.extend_from_slice(&dst_ip);
    pseudo.push(0);
    pseudo.push(6);
    pseudo.extend_from_slice(&(tcp_len as u16).to_be_bytes());
    pseudo.extend_from_slice(&segment.to_bytes());
    pseudo.extend_from_slice(&segment.payload);
    checksum(&pseudo)
}

#[derive(Debug, PartialEq)]
enum TcpState {
    Closed,
    SynSent,
    SynReceived,
    Established,
}

struct TcpClient {
    port: u16,
    state: TcpState,
    seq: u32,
    ack: u32,
    peer_seq: u32,
    window: VecDeque<TcpSegment>,
    window_size: usize,
}

impl TcpClient {
    fn new(port: u16) -> Self {
        Self {
            port,
            state: TcpState::Closed,
            seq: 1000,
            ack: 0,
            peer_seq: 0,
            window: VecDeque::new(),
            window_size: 5,
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
                    self.peer_seq = seg.seq;
                    self.ack = seg.seq + 1;
                    self.seq += 1;
                    self.state = TcpState::Established;
                    Some(TcpSegment::ack(self.port, seg.src_port, self.seq, self.ack))
                } else {
                    None
                }
            }
            TcpState::SynReceived => {
                if seg.ack_flag && !seg.syn {
                    self.state = TcpState::Established;
                }
                None
            }
            TcpState::Established => {
                if !seg.payload.is_empty() {
                    println!(
                        "Client {} received data: {:?}",
                        self.port,
                        String::from_utf8_lossy(&seg.payload)
                    );
                    self.ack = seg.seq + seg.payload.len() as u32;
                }
                None
            }
        }
    }

    fn send_data(&mut self, to: u16, msg: &str) -> Option<TcpSegment> {
        if self.window.len() < self.window_size {
            let seg = TcpSegment::data(self.port, to, self.seq, self.ack, msg.as_bytes().to_vec());
            self.seq += msg.len() as u32;
            self.window.push_back(seg.clone());
            Some(seg)
        } else {
            println!("Window full, waiting to send more");
            None
        }
    }

    fn ack_data(&mut self, ack_number: u32) {
        while let Some(front) = self.window.front() {
            let end_seq = front.seq;
            if end_seq <= ack_number {
                self.window.pop_front();
            } else {
                break;
            }
        }
    }
}

fn simulate_handshake_with_timeout() {
    let mut client_a = TcpClient::new(1000);
    let mut client_b = TcpClient::new(2000);

    let src_ip = [192, 168, 0, 1];
    let dst_ip = [192, 168, 0, 2];

    let mut tries = 0;
    let max_tries = 3;
    let mut handshake_done = false;

    while tries < max_tries && !handshake_done {
        println!("\nTry {}: sending SYN", tries + 1);
        let syn = client_a.send_syn(client_b.port);
        let syn = syn.with_checksum(src_ip, dst_ip);

        if let Some(syn_ack) = client_b.receive(&syn) {
            println!("SYN-ACK received");
            let ack = client_a.receive(&syn_ack).unwrap();
            client_b.receive(&ack);
            handshake_done = true;
        } else {
            println!("Timeout or no response. Retrying...");
        }

        tries += 1;
        thread::sleep(Duration::from_millis(500));
    }

    if handshake_done {
        println!("\n3-Way handshake completed!");

        if let Some(msg) = client_a.send_data(client_b.port, "Hello from A") {
            client_b.receive(&msg);
            client_a.ack_data(client_b.ack);
        }

        if let Some(reply) = client_b.send_data(client_a.port, "Hi from B") {
            client_a.receive(&reply);
            client_b.ack_data(client_a.ack);
        }
    } else {
        println!("Handshake failed after {} tries", max_tries);
    }
}

fn main() {
    simulate_handshake_with_timeout();
}
