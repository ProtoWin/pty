
extern crate redox_termios;

mod pty;


use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use pty::Pty;

use std::rc::Rc;
use std::sync::Arc;
use std::thread;

type Registry = Arc<Vec<Pty>>;

fn handle_client(registry: &mut Registry, stream: TcpStream) {
    // ...
}


fn main() {
    let mut registry: Registry = Arc::new(Vec::new());
    let listener = TcpListener::bind("127.0.0.1:2333").unwrap();
    for stream in listener.incoming() {
        let mut registry_clone = registry.clone();
        thread::spawn(move ||{
            handle_client(&mut registry_clone, stream.unwrap());
        });
    }
}