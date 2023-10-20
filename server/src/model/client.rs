use std::{
    io::BufReader,
    net::{SocketAddr, TcpStream},
};

pub struct Client {
    pub(crate) stream: TcpStream,
    pub(crate) addr: SocketAddr,
    pub(crate) reader: BufReader<TcpStream>,
}

#[derive(Default)]
pub struct Clients {
    pub(crate) clients: Vec<Client>,
}
