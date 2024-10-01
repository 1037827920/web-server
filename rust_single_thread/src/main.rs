use std::{
    // 帮助我们读取和写入数据
    // BufReader可以实现缓冲区读取，底层其实是基于std::io::Read实现，可以使用lines方法获取一个迭代器，可以对传输的内容流进行按行迭代读取，要使用该方法，需引入std::io::BufRead
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    fs,
};


fn main() {
    let listener = TcpListener::bind("localhost:8080").unwrap();
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        handle_connection(stream);
    }
}

/// # 函数作用
/// 处理连接：读取请求，回应请求
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    // 使用next而不是lines，因为我们只需要读取第一行，判断具体的request方法
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // 根据请求的不同，返回不同的响应
    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // write_all接收&[u8]类型作为参数，这里需要用as_bytes将字符串转换为字节数组
    stream.write_all(response.as_bytes()).unwrap();
}