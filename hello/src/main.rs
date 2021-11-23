use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::fs;
use hello::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = match ThreadPool::new(4) {
        Ok(p) => p,
        Err(e) => panic!("{}",e)
    };

    for stream in listener.incoming().take(2){
        let stream = stream.unwrap();

        pool.execute(|| {handle_connection(stream);});
    }
}

fn handle_connection(mut stream: TcpStream){
    let mut buffer = [0;1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n"; 

    let (status_lin, filename) = if buffer.starts_with(get) { 
        ("HTTP/1.1 200 OK","hello.html")
    }else if buffer.starts_with(sleep){
        std::thread::sleep(std::time::Duration::from_secs(5));
        ("HTTP/1.1 200 OK","hello.html")
    }else{
        ("HTTP/1.1 400 NOT FOUND","404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}\r\nContent-Lenght: {}\r\n\r\n{}"
                       ,status_lin,
                       contents.len(),contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

