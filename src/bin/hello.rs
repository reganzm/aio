use aio::runtime;
use aio::tcp::TcpListener;
use futures_util::stream::StreamExt;
use std::thread;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() {
    // set mode as UringPoll
    let kind = runtime::Kind::UringPoll;
    // get logical cpu number
    let cpus = num_cpus::get();
    println!("mode:{:?}, cpus:{}!", kind, cpus);
    let mut handles = Vec::new();
    // create cpus runtime
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
    let mut listener = TcpListener::bind("127.0.0.1", 12345);
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
