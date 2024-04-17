use crate::config::MCU_SOLID;
use crate::ddserror::DDSError;
use crate::log_func;
use lazy_static::lazy_static;

use colored::Colorize;
use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::result::Result;
use std::str;
use std::time::Duration;
use std::{
    io::{prelude::*, BufReader, Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
};

const READ_TIMEOUT: Duration = Duration::from_secs(5); //
const WRITE_TIMEOUT: Duration = Duration::from_secs(5); //
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10); //

lazy_static! {
    static ref ServerIP: SocketAddr = SocketAddr::new(IpAddr::V4(MCU_SOLID.ip()), MCU_SOLID.port());
}

/// `client_send` returns no longer `DDSError`, but `std::io::Result`
/// Because the `TcpStream::connect` returns it as well.
pub fn client_send(msg: String) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(ServerIP.clone())?;
    stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    stream.write(msg.as_bytes())?;
    log_func!("wait for response");
    let mut reader = BufReader::new(&stream);
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_until(b'\n', &mut buffer)?;
    println!(
        "receoved answer from ESP32 server {}",
        str::from_utf8(&buffer).unwrap_or("!!!!!")
    );
    log_func!();
    Ok(())
}

pub fn client_connect() -> std::io::Result<()> {
    let mut stream = TcpStream::connect(&ServerIP.clone())?;
    // let mut stream = TcpStream::connect_timeout(&ServerIP.clone(), CONNECT_TIMEOUT)?;
    let msg = "q";
    stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    stream.write(msg.as_bytes())?;
    log_func!("connecting...wait for response");
    let mut reader = BufReader::new(&stream);
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_until(b'\n', &mut buffer)?;
    println!(
        "receoved connection answer from ESP32 server {}, connection established.",
        str::from_utf8(&buffer).unwrap_or("!!!!!")
    );
    Ok(())
}

#[cfg(feature = "nets-debug")]
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from client");
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("Receoved request: {}", request);
    let response = "Hello, client!Jja;";
    stream
        .write(response.as_bytes())
        .expect("Failed to write to response to client ");
}

#[cfg(feature = "nets-debug")]
pub fn client_server() {
    log_func!("nets-debug mode enable, loopback test is enable");
    log_func!("start an tcp client to communiate with LoopBack");
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:8080");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
