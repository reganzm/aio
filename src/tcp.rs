use std::net::SocketAddr;
use socket2::{Domain, Socket, Type};
use crate::runtime;


pub struct TcpListener{
    host:&'static str,
    port:u32
}

impl TcpListener{
    pub fn bind(host:&'static str,port:u32)->runtime::Async<std::net::TcpListener>{
        Self{
            host,
            port
        }.create_listener()
    }
    fn create_listen_socket(&self) -> std::net::TcpListener {
        let addr: SocketAddr = format!("{0}:{1}", self.host, self.port).parse().unwrap();
        println!("{:?}", addr);
        let sock = Socket::new(
            match addr {
                SocketAddr::V4(_) => Domain::ipv4(),
                SocketAddr::V6(_) => Domain::ipv6(),
            },
            Type::stream(),
            None,
        )
        .unwrap();
        sock.set_reuse_address(true).unwrap();
        sock.set_reuse_port(true).unwrap();
        sock.set_nonblocking(true).unwrap();
        sock.bind(&addr.into()).unwrap();
        sock.listen(32768).unwrap();
        sock.into_tcp_listener()
    }
    pub fn create_listener(&self)->runtime::Async<std::net::TcpListener>{
        runtime::Async::<std::net::TcpListener>::new(self.create_listen_socket())
    }
}