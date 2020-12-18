use std::net;

pub struct TcpSocket {
    stream: net::TcpStream
}

impl super::Socket for TcpSocket {
    fn connect(addr: &str, port: usize) -> Result<Self, std::io::Error> {
        let full_addr = format!("{}:{}", addr, port);
        let mut stream = net::TcpStream::connect(&full_addr)?;

        Ok(TcpSocket { stream })
    }

    fn is_connected(&self) -> bool { true }

    fn send(&mut self, data: &[u8]) { unimplemented!() }
}