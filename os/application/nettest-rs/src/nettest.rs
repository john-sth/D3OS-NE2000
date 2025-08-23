// =============================================================================
// FILE        : nettest.rs
// AUTHOR      : Johann Spenrath
// DESCRIPTION : Rust Implementation of the nettest
//               application, based on the bachelor thesis
//               by Marcel Thiel
// =============================================================================
// TODO: add reference
//
// NOTES:
//
// =============================================================================
// DEPENDENCIES:
// =============================================================================
#![no_std]

extern crate alloc;

use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::str;
use core::time::Duration;
use network::UdpSocket;
use runtime::*;
use terminal::print;
use terminal::println;
use time;

#[derive(Debug)]
pub enum NetworkError {/* align with your definition */}

#[unsafe(no_mangle)]
fn main() {
    udp_send_packets("10.0.2.2", 12345, 1200, Some(1000), Some(100.0), None);
}

pub fn udp_send_packets(
    host: &str, // IP string, e.g. "10.0.2.2"
    port: u16,
    payload_size: usize,
    count: Option<usize>,
    pps: Option<f32>,
    duration_s: Option<f32>,
) {
}

/*
    /*let mut args = env::args().peekable();
    // the first argument is the program name, ignore it
    args.next();

    // check the next arguments for flags
    loop {
        match args.peek().map(String::as_str) {
            Some("-h") | Some("--help") => {
                println!(
                    "Usage:
    nettest host port

Examples:
    nc example.net 5678
        open a TCP connection to example.net:5678
    nc -u -l 0.0.0.0 1234
        bind to 0.0.0.0:1234, UDP"
                );
                return;
            }
            Some("-l") => {
                mode = Mode::Listen;
                args.next();
            }
            Some("-u") => {
                protocol = Protocol::Udp;
                args.next();
            }
            // now, we're finally past the options
            Some(_) => break,
            None => {
                println!("Usage: nc [-u] [-l] host port");
                return;
            }
        }
    }*/
}

pub fn udp_send_packets(
    host: &str, // IP string, e.g. "10.0.2.2"
    port: u16,
    payload_size: usize,
    count: Option<usize>,
    pps: Option<f32>,
    duration_s: Option<f32>,
) {
    if payload_size < 4 {
        println!("payload_size must be >= 4");
        return;
    }

    let dst_addr = parse_socket_addr(host, port).expect("invalid host");

    let sock = match UdpSocket::bind(dst_addr) {
        Ok(s) => s,
        Err(e) => {
            println!("bind failed: {:?}", e);
            return;
        }
    };

    // Handshake: send "Init", expect echo
    let init = b"Init";
    let _ = sock.send_to(init, dst_addr);
    let mut buf = [0u8; 1024];
    if let Ok((len, _peer)) = sock.recv_from(&mut buf) {
        if &buf[..len] == init {
            println!("Handshake OK");
        } else {
            println!("Unexpected handshake reply");
        }
    } else {
        println!("No echo from server after handshake");
    }

    let interval_ms = pps.map(|f| (1000.0 / f) as u64);
    let start_ms = time::systime();
    let mut sent = 0usize;
    let mut seq: u32 = 0;

    while {
        if let Some(d) = duration_s {
            let elapsed_sec: f32 = (time::systime() - start_ms).as_seconds_f32();

            elapsed_sec < d
        } else {
            true
        }
    } && count.map_or(true, |c| sent < c)
    {
        // make payload: 4-byte seq + zeros
        let mut packet = Vec::with_capacity(payload_size);
        packet.extend_from_slice(&seq.to_be_bytes());
        packet.extend_from_slice(&vec![0u8; payload_size - 4]);

        // send, handling backpressure
        loop {
            match sock.send_to(&packet, dst_addr) {
                Ok(_) => break,
                //Err(NetworkError::DeviceBusy) => {
                //    scheduler().sleep(1);
                //}
                Err(e) => {
                    println!("send error: {:?}", e);
                    return;
                }
            }
        }

        sent += 1;
        seq = seq.wrapping_add(1);

        if let Some(ms) = interval_ms {
            //scheduler().sleep(ms);
        }
    }

    // send "exit"
    let _ = sock.send_to(b"exit", dst_addr);

    println!("Sent {} packets", sent);
}

fn parse_socket_addr(host: &str, port: u16) -> Option<core::net::SocketAddr> {
    // Implement parsing logic for IpAddr and port, similar to resolve_hostname
    // For now, assume IPv4 literal
    host.parse().ok().map(|ip| core::net::SocketAddr::new(ip, port))
}
*/
