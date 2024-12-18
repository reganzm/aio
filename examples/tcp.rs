use aio::runtime;
use clap::clap_app;
use futures_util::stream::StreamExt;
use socket2::{Domain, Socket, Type};
use std::{env, net::SocketAddr, thread};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn create_listen_socket() -> std::net::TcpListener {
    let addr: SocketAddr = "[::]:12345".parse().unwrap();
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

const RESPONSE_HEADER: &str = "HTTP/1.1 200 OK";
const HELLO: &str = r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Async rust future</title>
  </head>
  <body>
    <p>hello,aio!</p>
  </body>
</html>
"#;
const HELLO_LEN: usize = HELLO.len();

async fn serve() {
    let mut listener = runtime::Async::<std::net::TcpListener>::new(create_listen_socket());
    while let Some(ret) = listener.next().await {
        if let Ok(mut stream) = ret {
            let f = async move {
                let mut buf = [0; 4096];
                match stream.read(&mut buf).await {
                    Ok(n) => {
                        let response = format!(
                            "{RESPONSE_HEADER}\r\nContent-Lenghth:{HELLO_LEN}\r\n\r\n{HELLO}"
                        );
                        if n == 0 || stream.write_all(response.as_bytes()).await.is_err() {
                            return;
                        }
                    }
                    Err(_) => {
                        return;
                    }
                }
            };
            runtime::spawn(f);
        }
    }
}

fn main() {
    let matches = clap_app!(greeter =>
        (@arg MODE: -m --mode +takes_value "specify I/O strategy, which can be: epoll, async, uringpoll, or hybrid")
    )
    .get_matches();

    let kind = if let Some(m) = matches.value_of("MODE") {
        match m {
            "epoll" => runtime::Kind::Epoll,
            "async" => runtime::Kind::Async,
            "uringpoll" => runtime::Kind::UringPoll,
            "hybrid" => runtime::Kind::Hybrid,
            _ => {
                println!("use 'epoll', 'async', 'uringpoll', or 'hybrid'");
                std::process::exit(1);
            }
        }
    } else {
        runtime::Kind::Epoll
    };

    let cpus = {
        env::var("RUSTMAXPROCS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(num_cpus::get)
    };

    println!("Hello, greeter-minimum: {:?} mode, ({} cpus)!", kind, cpus);

    let mut handles = Vec::new();
    for i in 0..cpus {
        let h = thread::spawn(move || {
            let ex = runtime::Runtime::new(kind).pin_to_cpu(i);

            ex.run(serve);
        });
        handles.push(h);
    }
    for h in handles {
        let _ = h.join();
    }
}
