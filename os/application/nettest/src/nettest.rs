// =============================================================================
// FILE        : nettest.rs
// AUTHOR      : Johann Spenrath
// DESCRIPTION : Rust Implementation of the nettest
//               application, based on the bachelor thesis
//               by Marcel Thiel
// Link        : https://github.com/hhuOS/hhuOS/tree/master/tools/nettest
// =============================================================================
//
// NOTES:
//
// =============================================================================
// DEPENDENCIES:
// =============================================================================

#![no_std]
extern crate alloc;

use alloc::vec;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use network::get_ip_addresses;
#[allow(unused_imports)]
use network::{TcpListener, TcpStream, UdpSocket, resolve_hostname};
use runtime::*;
use smoltcp::wire::{IpEndpoint, UDP_HEADER_LEN};
use terminal::{print, println, read::read};
use time;

#[derive(Debug)]
pub enum NetworkError {/* align with your definition */}
enum Socket {
    Udp(UdpSocket),
    //Tcp(TcpStream),
}

// =============================================================================
// disables the compiler's symbol name mangling,
// resulting in a globally visible symbol with a name that is not unique.
// =============================================================================
#[unsafe(no_mangle)]
fn main() {
    //println!(include_str!("banner.txt"));

    // =============================================================================
    // define a default how long packets should
    // be send in seconds
    // =============================================================================
    let time_interval: f64 = 20.0;

    // =============================================================================
    // define the default packet length
    // =============================================================================
    let packet_length: u16 = 64;

    // =============================================================================
    // open and bind udp socket to local address of the Host
    // =============================================================================
    let source_port = 1798;
    let local_socket_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), source_port);
    let socket_udp = UdpSocket::bind(local_socket_addr).expect("failed to open socket.");

    // =============================================================================
    // print the local ip address and listening port of the Host
    // =============================================================================
    let endpoint = IpEndpoint::new(smoltcp::wire::IpAddress::Ipv4(Ipv4Addr::new(10, 0, 2, 15)), source_port);
    //println!("[Local Endpoint: {}]", endpoint);

    // =============================================================================
    // create default destionation SocketAddr
    // =============================================================================
    let port = 2000;
    let dest_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 2)), port);

    run_udp_client(socket_udp, dest_addr, time_interval, packet_length);
    //run_udp_server(socket_udp, dest_addr);

    let tcp_sock = TcpListener::bind(dest_addr)
        .expect("failed to open socket")
        .accept()
        .expect("failed to accept connection");

    // connect the tcp socket to the server
    TcpStream::connect(dest_addr).expect("failed to open socket");
    //tcp_send_traffic(tcp_sock, dest_addr, packet_length, time_interval)
}

// =============================================================================
// function run_udp_client
// =============================================================================
// Sends "Init\n" to server, waits for an "Init\n" response,
// calls send_traffic function, then returns.
// =============================================================================
pub fn run_udp_client(socket: UdpSocket, dest_addr: SocketAddr, time_interval: f64, packet_length: u16) -> Result<(), &'static str> {
    // =============================================================================
    // define variables
    // =============================================================================

    // define buffer size
    let mut buf = [0u8; 1024];

    // define Init message for connection
    let init_msg = b"Init\n";

    // write message into buffer
    buf[..init_msg.len()].copy_from_slice(init_msg);

    // =============================================================================
    // Step 1: send the init message to the receiving host
    // =============================================================================
    println!("[UDP: sending Init to {}.]", dest_addr);
    UdpSocket::send_to(&socket, init_msg, dest_addr).expect("failed to send over UDP");

    // =============================================================================
    // Step 2: Poll for response
    // =============================================================================
    let deadline = time::systime().as_seconds_f64() + 5.0; // 5s timeout
    println!("[Waiting for server reply.]");
    // ======================================
    // loop until a socket has been received
    // or the timeout is reached
    // ======================================
    loop {
        if time::systime().as_seconds_f64() > deadline {
            println!("[timeout waiting for Init response]");
            return Err("timeout waiting for Init response");
        }
        // get the length of the reply
        let len = UdpSocket::recv_from(&socket, &mut buf).expect("failed to receive over UDP").0;

        // ======================================
        // if a reply has been received and is not
        // empty convert to string
        // ======================================
        if len > 0 {
            let ack = str::from_utf8(&buf[0..len]).expect("failed to parse received string");
            println!("UDP: received ACK {:?}", ack);
            // ======================================
            // if the ACK equals the init message
            // go to the next step and
            // send packets in a defined interval
            // ======================================
            if ack == str::from_utf8(init_msg).unwrap() {
                println!("Received expected Init response");
                time_interval =
                //udp_send_traffic(socket, dest_addr, time_interval, packet_length);
                return Ok(());
            } else {
                println!("Unexpected data: {:?}", ack);
                return Err("Wrong Init response");
            }
        }
    }
}

// =============================================================================
// function udp_send_traffic:
// =============================================================================
// initated by run_udp_client
// sends a burst of packets in the defined time interval
// =============================================================================
pub fn udp_send_traffic(socket: UdpSocket, dest_addr: SocketAddr, time_interval: f64, packet_length: u16) -> Result<(), &'static str> {
    // =============================================================================
    // define variables
    // =============================================================================
    // save how many bytes have been sent in one second
    let mut bytes_sent_in_interval: u128 = 0;
    let mut interval_counter: usize = 0;
    // save the number of packets send in the time period
    let mut packets_send: u128 = 0;

    // ======================================
    // create the packet
    // ======================================
    let mut buf = vec![0; packet_length as usize];
    let datagram: &mut [u8] = &mut buf;

    // define the time for exit
    //let test_finish_time = time::systime().as_seconds_f64() + time_interval;
    let test_finish_time = time::systime() + time_interval;
    // define counter variable seconds_passed for each passing second
    //let mut seconds_passed = time::systime().as_seconds_f64() + 1.0;
    let mut seconds_passed = time::systime() + 1.0;
    //jprintln!("//=============================================================//");
    println!(
        "  [Start Time: {}] |=========| [End Time: {}]  ",
        time::systime().as_seconds_f64(),
        test_finish_time
    );
    //println!("//=============================================================//");

    // loop until the time interval has been reached
    //while time::systime().as_seconds_f64() < test_finish_time {
    while time::systime() < test_finish_time {
        // count each packet being send
        packets_send += 1;
        // ======================================
        // save the current number in the
        // current packet this is later being
        // read by the server for verifying
        // duplicated and out of order packets
        // ======================================
        datagram[0] = ((packets_send >> 24) & 0xff) as u8;
        datagram[1] = ((packets_send >> 16) & 0xff) as u8;
        datagram[2] = ((packets_send >> 8) & 0xff) as u8;
        datagram[3] = (packets_send & 0xff) as u8;

        // send the packet
        UdpSocket::send_to(&socket, &datagram, dest_addr).expect("failed to send over UDP");

        // ======================================
        // count bytes send in each second
        // by adding the packet length
        // of each sent packet
        // ======================================
        bytes_sent_in_interval += packet_length as u128;

        // ======================================
        // if a second passes print out the
        // current stats in the terminal
        // ======================================
        //if seconds_passed < time::systime().as_seconds_f64() {
        if seconds_passed < time::systime() {
            println!(
                "[{:?} - {:?}] : [{} KB/s]",
                interval_counter,
                interval_counter + 1,
                (bytes_sent_in_interval as f64) / 1000.0
            );
            // update counters after a passed second
            interval_counter += 1;
            bytes_sent_in_interval = 0;
            seconds_passed += 1.0;
        }
    }
    // ======================================
    // after the end of the time_inverval send
    // an exit message to the server
    // to signal end of transmission
    // ======================================
    let end_datagram: &[u8] = b"exit\n";
    UdpSocket::send_to(&socket, &end_datagram, dest_addr).expect("failed to send end message");

    // ======================================
    // get the total number of bytes send in
    // the defined time interval
    // ======================================
    let sent_bytes = packet_length as u128 * packets_send;
    // ======================================
    // print the end result to
    // the terminal screen
    // ======================================
    println!("==> [ reached finish time! ]");
    println!("");
    println!("//====================================================//");
    println!("    [Number of transmitted packets]  ==> {}", packets_send);
    println!("    [total Bytes transmitted]        ==> {}", sent_bytes);
    println!("    [Average KB/s]                   ==> {} KB/s", (sent_bytes as f64 / time_interval) / 1000.0);
    println!("//====================================================//");
    return Ok(());
}

// =============================================================================
// function run_udp_server:
// =============================================================================
// waits for an incoming connection request,
// if a request, is made, a reply is sent back
// and the receive_traffic function is initated
// =============================================================================
pub fn run_udp_server(socket: UdpSocket, dest_addr: SocketAddr) -> Result<(), &'static str> {
    // =============================================================================
    // define variables
    // =============================================================================
    let packet_length: u16 = 64;
    let init_msg = b"Init\n";

    // wait for a connection request from the client
    let mut buf = vec![0; packet_length as usize];

    println!("Server starting up...");
    println!("waiting for Init request.");

    // =============================================================================
    // loop until an init request by a client
    // has been made
    // =============================================================================
    loop {
        let (len, sender) = UdpSocket::recv_from(&socket, &mut buf).expect("failed to receive over UDP");

        // ==============================================
        // if the request contains something,
        // convert to string and check if
        // the init message is correct
        // if yes, acknowledge the request by sending
        // the init message back and
        // start the udp_receive_traffic function
        // ==============================================
        if len > 0 {
            let request_msg = str::from_utf8(&buf[0..len]).expect("failed to parse received string");
            print!("{request_msg}");
            println!("UDP: received {:?} from {:?}", request_msg, sender.ip());
            if request_msg == str::from_utf8(init_msg).unwrap() {
                println!("Received expected Init response");
                UdpSocket::send_to(&socket, init_msg, dest_addr).expect("failed to send init message.");
                udp_receive_traffic(socket);
                return Ok(());
            } else {
                println!("Unexpected data: {:?}", request_msg);
                return Err("Wrong Init response");
            }
        }
    }
}

// =============================================================================
// function udp_receive_traffic:
// =============================================================================
// function processes in a loop incoming packets
// and checks if a packet has been duplicated or
// is out of order
// every second the current bytes received are printed to the terminal
// the function ends if the server sends an exit request
// and prints out statistics about the received traffic
// =============================================================================
pub fn udp_receive_traffic(sock: UdpSocket) -> Result<(), &'static str> {
    // =============================================================================
    // define variables
    // =============================================================================
    let mut packets_received: u32 = 0;
    let mut packets_out_of_order: u32 = 0;
    let mut duplicated_packets: u32 = 0;
    let mut current_packet_number: u32 = 0;
    let mut previous_packet_number: u32 = 0;
    let mut interval_counter: usize = 0;
    let mut bytes_received: usize = 0;
    let mut bytes_received_in_interval: usize = 0;
    let mut bytes_received_total: usize = 0;
    let mut seconds_passed = 0.0;
    // define exit_msg
    let exit_msg = b"exit\n";

    //this lead to an error in which no packet was received now it works, when just passing the socket
    // from the calling method
    //let source_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 15));
    //let dest_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 2));
    //let sock = network::open_udp();

    // =============================================================================
    // define a buffer in which the received packetgets saved to
    // =============================================================================
    let mut buf = vec![0; 2048];

    // =============================================================================
    // wait for the reception of the first packet
    // then start the timer and the loop
    // =============================================================================
    let deadline = time::systime().as_seconds_f64() + 5.0; // set 5s deadline for packet to arrive
    println!("waiting for first packet ");
    loop {
        if time::systime().as_seconds_f64() > deadline {
            return Err("timeout waiting for first packet ");
        }

        let (len, sender) = UdpSocket::recv_from(&sock, &mut buf).expect("failed to parse first packet.");
        // rec_data is of type u8 which would overflow -> cast recv_data in previous and current to u32
        // get the received payload of the packet from the buffer
        let recv_data = &buf[..len];

        println!("UDP: received first packet from {:#?}:{:#?}", sender.ip(), sender.port());

        // =============================================================================
        // read the first 4 bytes of the packet payload which contain
        // the number of the nth packet, which has been sent by the client
        // =============================================================================
        //if recv_data.len() >= 4 {
        previous_packet_number = u32::from_be_bytes([recv_data[0], recv_data[1], recv_data[2], recv_data[3]]);
        //} else {
        //    println!("Received packet too short: len = {}", recv_data.len());
        //    continue; // or handle error
        //}

        println!("");
        println!("//======================================================//");
        println!("  <<Start Time: {}    >>", time::systime().as_seconds_f64());
        println!("//======================================================//");
        // =============================================================================
        // initalize the counter variables
        // =============================================================================
        seconds_passed = time::systime().as_seconds_f64() + 1.0;
        packets_received += 1;
        bytes_received_in_interval = len;
        break;
    }

    // =============================================================================
    // receive packets in a loop
    // until an exit msg is sent
    // =============================================================================
    loop {
        let len = UdpSocket::recv_from(&sock, &mut buf).expect("Failed to parse Packet.").0;
        let recv_data = &buf[..len];
        // exit condition
        if recv_data == exit_msg {
            break;
        }

        // count number of packets received
        packets_received += 1;

        // =============================================================================
        // read the first 4 bytes of the packet payload which contain
        // the number of the nth packet, which has been sent by the client
        // =============================================================================

        // cast to u8 because of overflow error thrown by compiler
        //if recv_data.len() >= 4 {
        current_packet_number = u32::from_be_bytes([recv_data[0], recv_data[1], recv_data[2], recv_data[3]]);
        //}
        //current_packet_number =
        //(recv_data[0] >> 24 & 0xFF) as u32 + (recv_data[1] >> 16 & 0xFF) as u32 + (recv_data[2] >> 8 & 0xFF) as u32 + (recv_data[3] & 0xFF) as u32;

        // =============================================================================
        // if the first 4 bytes of the previous and current packet
        // are equal then the packet had been retransmitted
        // =============================================================================
        if current_packet_number == previous_packet_number {
            duplicated_packets += 1;
        // =============================================================================
        // check if the packet has been received out of order
        // =============================================================================
        } else if current_packet_number != (previous_packet_number + 1) || current_packet_number < previous_packet_number {
            packets_out_of_order += 1;
        }

        // update values, count the received bytes
        previous_packet_number = current_packet_number;
        bytes_received_in_interval += len;

        // =============================================================================
        // if a second has passed, print out the number of bytes which
        // have been received in the current second
        // =============================================================================

        if seconds_passed < time::systime().as_seconds_f64() {
            println!(
                "[{} - {}] : [{} KB/s]",
                interval_counter,
                interval_counter + 1,
                bytes_received_in_interval / 1000
            );
            interval_counter += 1;
            bytes_received += bytes_received_in_interval;
            bytes_received_in_interval = 0;
            seconds_passed += 1.0;
        }
    }
    bytes_received += bytes_received_in_interval;

    println!(
        "[{} - {}] : [{} KB/s]",
        interval_counter,
        interval_counter + 1,
        bytes_received_in_interval / 1000
    );
    println!("");
    println!("//==============================================================//");
    println!("  [Received exit message from Client: End of reception!]");
    println!("//==============================================================//");
    println!("");
    println!("//==============================================================//");
    println!("  [Number of packets received] ==> {}", packets_received);
    println!("  [Total bytes received]       ==> {}", bytes_received_total);
    println!("  [Bytes received]             ==> {} KB/s", bytes_received / 1000);
    println!("  [Average Bytes received]     ==> {} KB/s", (bytes_received / (interval_counter + 1)) / 1000);
    println!("  [packets out of order]       ==> {}", packets_out_of_order / packets_received);
    println!("  [duplicated packets]         ==> {}", duplicated_packets);
    println!("//==============================================================//");
    return Ok(());
}

//pub fn tcp_send_traffic(sock: TcpStream, addr: SocketAddr, packet_length: u16, time_interval: f64) -> Result<(), &'static str> {
pub fn tcp_send_traffic(sock: TcpStream, addr: SocketAddr, packet_length: u16, time_interval: f64) {
    let mut packets_send = 0;
    let mut interval_counter = 0;
    let mut bytes_sent_in_interval: u128 = 0;
    // create tcp socket
    let tcp_sock = TcpListener::bind(addr)
        .expect("failed to open socket")
        .accept()
        .expect("failed to accept connection");

    // connect the tcp socket to the server
    TcpStream::connect(addr).expect("failed to open socket");

    // ======================================
    // create the packet
    // ======================================
    let mut buf = vec![0; packet_length as usize];
    let datagram: &mut [u8] = &mut buf;

    // define the time for exit
    let test_finish_time = time::systime().as_seconds_f64() + time_interval;
    // define counter variable seconds_passed for each passing second
    let mut seconds_passed = time::systime().as_seconds_f64() + 1.0;
    //jprintln!("//=============================================================//");
    println!(
        "  [Start Time: {}] |=========| [End Time: {}]  ",
        time::systime().as_seconds_f64(),
        test_finish_time
    );
    //println!("//=============================================================//");

    let mut buf = [0u8; 1024];
    TcpStream::write(&tcp_sock, &datagram).expect("failed to send char");
    // loop until the time interval has been reached
    while time::systime().as_seconds_f64() < test_finish_time {
        // count each packet being send
        packets_send += 1;
        // ======================================
        // save the current number in the
        // current packet this is later being
        // read by the server for verifying
        // duplicated and out of order packets
        // ======================================
        datagram[0] = ((packets_send >> 24) & 0xff) as u8;
        datagram[1] = ((packets_send >> 16) & 0xff) as u8;
        datagram[2] = ((packets_send >> 8) & 0xff) as u8;
        datagram[3] = (packets_send & 0xff) as u8;

        // ======================================
        // count bytes send in each second
        // by adding the packet length
        // of each sent packet
        // ======================================
        bytes_sent_in_interval += packet_length as u128;

        // ======================================
        // if a second passes print out the
        // current stats in the terminal
        // ======================================
        if seconds_passed < time::systime().as_seconds_f64() {
            println!(
                "[{:?} - {:?}] : [{} KB/s]",
                interval_counter,
                interval_counter + 1,
                (bytes_sent_in_interval as f64) / 1000.0
            );
            // update counters after a passed second
            interval_counter += 1;
            bytes_sent_in_interval = 0;
            seconds_passed += 1.0;
        }
    }
    // ======================================
    // print the end result to
    // the terminal screen
    // ======================================
    // ======================================
    // get the total number of bytes send in
    // the defined time interval
    // ======================================
    let sent_bytes = packet_length as u128 * packets_send;
    println!("==> [ reached finish time! ]");
    println!("");
    println!("//====================================================//");
    println!("    [Number of transmitted packets]  ==> {}", packets_send);
    println!("    [total Bytes transmitted]        ==> {}", sent_bytes);
    println!("    [Average KB/s]                   ==> {} KB/s", (sent_bytes as f64 / time_interval) / 1000.0);
    println!("//====================================================//");
}
