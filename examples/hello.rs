//! Echo example.
//! Use `nc 127.0.0.1 30000` to connect.

use aio::{excutor::Executor, tcp::TcpListener};
use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() {
    let ex = Executor::new();
    ex.block_on(serve);
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
    let mut listener = TcpListener::bind("127.0.0.1:12345").unwrap();
    while let Some(ret) = listener.next().await {
        if let Ok((mut stream, addr)) = ret {
            let f = async move {
                let mut buf = [0; 4096];
                match stream.read(&mut buf).await {
                    Ok(n) => {
                        let response = format!(
                            "{RESPONSE_HEADER}\r\nContent-Lenght:{HELLO_LEN}\r\n\r\n\n{HELLO}"
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
            Executor::spawn(f);
        }
    }
}
