//! Echo example.
//! Use `nc 127.0.0.1 30000` to connect.

use aio::{excutor::Executor, tcp::TcpListener};
use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() {
    let ex = Executor::new();
    // block_on function will block the current thread until the future is ready
    // and return the result of the future
    ex.block_on(serve);
}

/// serve is a simple tcp server that sends a hello message to the browser client
async fn serve() {
    // bind the server to the 127.0.0.1:12345
    let mut listener = TcpListener::bind("127.0.0.1:12345").unwrap();
    // accept the incoming connection until forever
    while let Some(ret) = listener.next().await {
        if let Ok((mut stream, _addr)) = ret {
            // under is a async block that reads the incoming data and sends the hello message to client
            let f = async move {
                // read the incoming data
                let mut buf = [0; 4096];
                match stream.read(&mut buf).await {
                    Ok(n) => {
                        // response message : header + conent-length + content
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
            // put the future into the task queue
            Executor::spawn(f);
        }
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
