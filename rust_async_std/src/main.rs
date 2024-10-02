use std::{
    fs,
    time::Duration,
};
extern crate async_std;
use async_std::{
    net::{TcpListener, TcpStream},
    io::{prelude::*, BufReader},
    task,
};
use futures::StreamExt;

#[async_std::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    
    listener
        .incoming()
        .for_each_concurrent(None, |tcpstream| async move {
            let tpcstream = tcpstream.unwrap();
            handle_connection(tpcstream).await;
        })
        .await;
}
/// # 函数作用
/// 处理连接：读取请求，回应请求
async fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    // 使用next而不是lines，因为我们只需要读取第一行，判断具体的request方法
    let request_line = buf_reader.lines().next().await.unwrap().unwrap();

    // 根据请求的不同，返回不同的响应
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"), // 请求 / 资源
        "GET /sleep HTTP/1.1" => { // 请求 /sleep 资源
            // 没有使用std::thread::sleep进行睡眠，原因是该函数是阻塞的，它会让当前线程陷入睡眠中，导致其他任务无法继续运行
            task::sleep(Duration::from_secs(5)).await;
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // write_all接收&[u8]类型作为参数，这里需要用as_bytes将字符串转换为字节数组
    stream.write_all(response.as_bytes()).await.unwrap();
}
