use clap::clap_app;
use futures_util::stream::StreamExt;
use tcp::TcpListener;
use std::{env, thread};

mod runtime;
mod tcp;

async fn serve() {
    let mut listener = TcpListener::bind("127.0.0.1:30000").unwrap();
    while let Some(ret) = listener.next().await {
        if let Ok((mut stream, addr)) = ret {
            println!("accept a new connection from {} successfully", addr);
            let f = async move {
                let mut buf = [0; 4096];
                loop {
                    match stream.read(&mut buf).await {
                        Ok(n) => {
                            if n == 0 || stream.write_all(&buf[..n]).await.is_err() {
                                return;
                            }
                        }
                        Err(_) => {
                            return;
                        }
                    }
                }
            };
            //Executor::spawn(f);
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
