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
//#![no_std]
//
//extern crate alloc;
//
//use alloc::ffi::CString;
//use alloc::string::String;
//use alloc::vec;
//use alloc::vec::Vec;
//use core::str;
//use core::time::Duration;
//use network::UdpSocket;
//use runtime::*;
//use terminal::print;
//use terminal::println;

#![no_std]
extern crate alloc;

use core::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::from_utf8,
};

use alloc::string::String;
use network::{TcpListener, TcpStream, UdpSocket, resolve_hostname};
#[allow(unused_imports)]
use runtime::*;
use smoltcp::wire::IpEndpoint;
use terminal::{print, println, read::read};
use time;

#[derive(Debug)]
pub enum NetworkError {/* align with your definition */}
enum Socket {
    Udp(UdpSocket),
    Tcp(TcpStream),
}

#[unsafe(no_mangle)]
fn main() {
    run_udp_client("10.0.2.2", 12345);
}

/*pub fn udp_send_traffic(n: usize, interval: u16, packet_length: u16) -> Result<(), &'static str> {
    let dst_port = 12345;
    let dst_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 2));
    let mut bytes_sent_in_interval = 0;
    let mut interval_counter: usize = 0;
    network::bind_udp(sock, dst_ip, dst_port).expect("socket bind failed");

    //let dst_ip = Ipv4Address::new(10, 0, 2, 2);
    let mut packet_number: u32 = 0;
    //let mut datagram = [0u8; 10];
    //let datagram = b"data\n";
    let mut buf = vec![0; packet_length as usize];
    let datagram: &mut [u8] = &mut buf;

    let test_finish_time = timer().systime_ms() + 20_000;
    let mut seconds_passed = timer().systime_ms() + 1_000;
    info!("Start: {} - End: {}", timer().systime_ms(), test_finish_time);
    info!("--------------------------------------------------------");

    //for _ in 0..n {
    while timer().systime_ms() < test_finish_time {
        packet_number += 1;
        datagram[0] = ((packet_number >> 24) & 0xff) as u8;
        datagram[1] = ((packet_number >> 16) & 0xff) as u8;
        datagram[2] = ((packet_number >> 8) & 0xff) as u8;
        datagram[3] = (packet_number & 0xff) as u8;
        // retry until queued; the poll thread will drain TX between retries
        //loop {
        // catch error buffer full by givinsmoltcp::wire::IpAddress::Ipv4(g the )poll method more time
        match network::send_datagram(sock, dst_ip, dst_port, datagram) {
            Ok(()) => {
                //break;
            }
            Err(SendError::BufferFull) => {
                info!("Buffer full");
                // give the poll method time to flush and to finish ARP, then retry
                //scheduler().sleep(1);
            }
            Err(e) => panic!("(UDP Send Test) send failed: {e:?}"),
        }

        bytes_sent_in_interval += packet_length;

        // if a second passes print the current stats at the screen
        if seconds_passed < timer().systime_ms() {
            info!(
                "{} - {} : {} KB/s",
                interval_counter,
                interval_counter + 1,
                (bytes_sent_in_interval as f32) / 1000.0
            );
            interval_counter += 1;
            bytes_sent_in_interval = 0;
            seconds_passed += 1_000;
        }
        //network::send_datagram(sock, dst_ip, dst_port, datagram);
        //}
        // light pacing so the CPU doesn't get hoged
        network::poll_ne2000_tx();
        //scheduler().sleep(20);
    }
    // after the end of the inverval send an exit message to the server
    let end_datagram: &[u8] = b"exit\n";
    match network::send_datagram(sock, dst_ip, dst_port, end_datagram) {
        Ok(()) => {}
        Err(SendError::BufferFull) => {
            info!("Buffer full");
        }
        Err(e) => panic!("(UDP Send Test) send failed: {e:?}"),
    }

    // get the total number of bytes send in the defined time interval
    let send_bytes = packet_length as u32 * packet_number;
    info!("--------------------------------------------------------");
    info!("Packets transmitted : {}", packet_number);
    info!("Bytes transmitted: {}", send_bytes);
    info!("Average: {} KB/s", (send_bytes / interval as u32) / 1000);
    return Ok(());
}*/

//use crate::{close_socket, get_ip_addresses, open_udp, receive_datagram, send_datagram};

/// Sends "Init\n" to server, waits for an "Init\n" response, then returns.
pub fn run_udp_client(server: &str, port: u16) -> Result<(), &'static str> {
    // 1) Resolve server address
    //let dest_ip = parse_or_resolve(server).ok_or("DNS resolution failed")?;
    //let dest_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 2));
    let interval: u16 = 2;
    let packet_length: u16 = 64;
    let source_port = 1798;
    let dest_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 2)), port);

    // define the packet
    let msg = b"Init\n";
    let mut buf = [0u8; 1024];
    // write message into buffer
    buf[..msg.len()].copy_from_slice(msg);

    // 2) Open a UDP socket
    // open and bind udp socket to local address
    let source_ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), source_port);
    let socket = Socket::Udp(UdpSocket::bind(source_ip).expect("failed to open socket."));

    //let endpoint = IpEndpoint::new(source_ip.ip(), port);
    //println!("Local Endpoint: {}", endpoint);

    // 3) Send "Init\n"
    println!("UDP: sending Init to 127.0.0.1:12345");
    //println!("UDP: sending Init to {}", endpoint);
    //Socket::Udp.send_to(string.as_bytes(), dest_ip);
    //Socket::Udp(sock) => sock.send_to(buf, dest_ip);
    match &socket {
        Socket::Udp(sock) => {
            sock.send_to(msg, dest_addr).expect("failed to send over UDP");
        }
        Socket::Tcp(sock) => {
            //sock.write_all(buf).expect("failed to send over TCP");
        }
    }

    // 4) Poll for response
    let deadline = time::systime().as_seconds_f64() + 5.0; // 5s timeout
    println!("Waiting for server reply");
    loop {
        if time::systime().as_seconds_f64() > deadline {
            println!("timeout waiting for Init response");
            return Err("timeout waiting for Init response");
        }

        let len = match &socket {
            Socket::Udp(sock) => sock.recv_from(&mut buf).expect("failed to receive over UDP").0,

            Socket::Tcp(sock) => 0,
        };
        if len > 0 {
            let text = str::from_utf8(&buf[0..len]).expect("failed to parse received string");
            print!("{text}");
            println!("UDP: received {:?}", text);
            if text == str::from_utf8(msg).unwrap() {
                println!("Received expected Init response");
                //udp_send_traffic(n, interval, packet_length);
                return Ok(());
            } else {
                println!("Unexpected data: {:?}", text);
                return Err("Wrong Init response");
            }
        }
    }
    //info!("polling for response");
    // Let other threads run / allow network stack to poll
    //scheduler().sleep(10);
    //network::poll_sockets_ne2k();
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
