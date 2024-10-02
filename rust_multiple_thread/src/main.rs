use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use rust_multiple_thread::ThreadPool;

fn main() {
	let listener = TcpListener::bind("localhost:8080").unwrap();
	// 首先创建一个包含4个线程的线程池
    let pool = ThreadPool::new(4);
    
    // listener.incoming().take(2)表示只处理前两个请求，可以用来测试线程池能够正常进行资源清理
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        // 分发执行请求
        pool.execute(|| {
            handle_connection(stream)
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
	let buf_reader = BufReader::new(&mut stream);
    // 使用next而不是lines，因为我们只需要读取第一行，判断具体的request方法
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    
	// match方法不会像之前的方法那样自动做引用或解引用，因此我们需要显式调用
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"), // 请求 / 资源
        "GET /sleep HTTP/1.1" => { // 请求 /sleep 资源
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    // 读取文件内容
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    // 格式化HTTP Response
    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // 将response写入stream
    stream.write_all(response.as_bytes()).unwrap();
}