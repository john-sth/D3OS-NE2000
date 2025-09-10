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

use alloc::string::String;

use alloc::vec;
use chrono::{self, Duration, TimeDelta};
use concurrent::thread;
use core::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    result,
};
use network::NetworkError;
#[allow(unused_imports)]
use network::{TcpListener, TcpStream, UdpSocket, resolve_hostname};
use runtime::*;
use smoltcp::wire::*;
use syscall::return_vals::Errno;
use terminal::{print, println};
use time;

enum Protocol {
    Udp,
    Tcp,
}
enum Socket {
    Udp(UdpSocket),
    Tcp(TcpStream),
}
enum Mode {
    Listen,
    Connect,
}

enum Error {
    Timeout(String),
    WrongInitResponse(String),
}

// =============================================================================
// disables the compiler's symbol name mangling,
// resulting in a globally visible symbol with a name that is not unique.
// =============================================================================
#[unsafe(no_mangle)]
fn main() {
    println!(include_str!("banner.txt"));

    // =============================================================================
    // parse the arguments
    // taken from nc.rs
    // =============================================================================
    let mut server_mode = false;
    let mut args = env::args().peekable();
    // the first argument is the program name, ignore it
    args.next();

    let mut mode = Mode::Connect;
    let mut protocol = Protocol::Tcp;

    // ======================================
    // check the next arguments for flags
    // ======================================
    loop {
        match args.peek().map(String::as_str) {
            Some("-h") | Some("--help") => {
                println!(
                    "Usage:
    nettest [-u] [-l] HOST PORT [REMOTE_HOST] [REMOTE_PORT] [duration] [packet_length]\n
    -u: Specify the protocol (UDP/TCP) \n
    -l: Server mode, listen on HOST:PORT. \n
    duration : for client mode, specify time for how long to send packets.\n
    packet_length : for client mode, specify the preferred length of the packets, which should be transmitted."
                );
                return;
            }
            Some("-l") => {
                mode = Mode::Listen;
                server_mode = true;
                args.next();
            }
            Some("-u") => {
                protocol = Protocol::Udp;
                args.next();
            }
            // now, we're finally past the options
            Some(_) => break,
            None => {
                println!("Usage: nettest [-u] [-l] host port remote_host remote_port duration packet_length");
                return;
            }
        }
    }

    // ======================================
    // the next arguments should be host and port
    // for listen, this is the address and port to bind to
    // for connect, this is the remote host to connect to
    // ======================================
    let addr = if let Some(host) = args.next()
        && let Some(port_str) = args.next()
    {
        // just take the first IP address
        let ip = resolve_hostname(&host).into_iter().next().unwrap();
        let port: u16 = port_str.parse().expect("[failed to parse port]");
        SocketAddr::new(ip, port)
    } else {
        println!("Usage: nettest [-u] [-l] host port remote_host remote_port duration packet_length");
        return;
    };

    // ======================================
    // parse the remote host and port
    // ======================================
    let addr_remote = if let Some(host_rem) = args.next()
        && let Some(port_rem_str) = args.next()
    {
        // just take the first IP address
        let ip_rem = resolve_hostname(&host_rem).into_iter().next().unwrap();
        let port_rem: u16 = port_rem_str.parse().expect("[failed to parse port.]");
        SocketAddr::new(ip_rem, port_rem)
    } else if server_mode {
        // if the application runs in server mode just fill with dummy values
        // since the address doesnt get used
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
    } else {
        println!("Usage: nettest [-u] [-l] host port remote_host remote_port duration packet_length");
        return;
    };

    // =============================================================================
    // define a default how long packets should
    // be send in seconds
    // =============================================================================
    let time_interval = if let Some(time) = args.next() {
        let seconds = time.parse().expect("[failed to parse number of seconds.]");
        chrono::TimeDelta::seconds(seconds)
    } else {
        chrono::TimeDelta::seconds(20)
    };

    // =============================================================================
    // define the default packet length
    // =============================================================================
    let payload_length = if let Some(length) = args.next() {
        let size = length.parse().expect("[failed to parse payload length]");
        size
    } else {
        1024
    };
    // =============================================================================
    // open and bind udp socket to local address of the Host
    // =============================================================================
    //let socket_udp = UdpSocket::bind(addr).expect("[failed to open socket.]");

    //match mode {
    //    Mode::Listen => return run_udp_server(socket_udp).expect("[[failed to start udp server]"),
    //    Mode::Connect => return run_udp_client(socket_udp, addr_remote, time_interval, payload_length).expect("[failed to start udp client.]"),
    //}
    //return tcp_send_traffic(dest_addr, payloaddest_addr_length, time_interval);

    let socket = match mode {
        Mode::Listen => match protocol {
            Protocol::Udp => Socket::Udp(UdpSocket::bind(addr).expect("failed to open socket")),
            Protocol::Tcp => Socket::Tcp(
                TcpListener::bind(addr)
                    .expect("failed to open socket")
                    .accept()
                    .expect("failed to accept connection"),
            ),
        },
        Mode::Connect => match protocol {
            Protocol::Udp => Socket::Udp(UdpSocket::bind(addr).expect("failed to open socket")),
            Protocol::Tcp => Socket::Tcp(TcpStream::connect(addr_remote).expect("failed to open socket")),
        },
    };

    match socket {
        Socket::Udp(sock) => match mode {
            Mode::Listen => return run_udp_server(sock).expect("[failed to start udp server]"),
            Mode::Connect => return run_udp_client(sock, addr_remote, time_interval, payload_length).expect("[failed to start udp client.]"),
        },
        Socket::Tcp(sock) => match mode {
            Mode::Listen => return run_tcp_server(sock).expect("[TCP Server failed to start"),
            Mode::Connect => return run_tcp_client(sock, time_interval, payload_length).expect("[TCP Client failed to start!]"),
        },
    };

    // connect the tcp socket to the server
    //TcpStream::connect(addr_remote).expect("[failed to open socket]");
    //tcp_send_traffic(tcp_sock, dest_addr, packet_length, time_interval)
}

pub fn run_tcp_server(socket: TcpStream) -> Result<()> {
    println!("TCP Server starting up...");
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
    let mut seconds_passed = TimeDelta::zero();
    // use this variable to print out the received payload length in the
    // end results
    let mut packet_payload_length = 0;
    // define exit_msg
    //let exit_msg = b"";
    let mut exit = false;
    // define a buffer in which the received packetgets saved to
    let mut buf = vec![0; 2048];

    println!("[nettest: server listening! Send 'exit' to leave.]");

    // =============================================================================
    // wait for the reception of the first packet
    // then start the timer and the loop
    // =============================================================================
    let deadline = time::systime() + Duration::seconds(5); // set 5s deadline for packet to arrive
    println!("[waiting for first packet...]");
    loop {
        if time::systime() > deadline {
            panic!("[timeout waiting for first packet]");
        }

        let result_size = TcpStream::read(&socket, &mut buf).expect("[failed to receive datagram!]");
        let recv_data = &buf[..result_size];
        if recv_data.len() >= 4 {
            println!("");
            println!("[======================================================]");
            println!("  [  Start Time: {}  ]", time::systime().as_seconds_f64());
            println!("[======================================================]");
            println!("");
            packet_payload_length = result_size;
            // =============================================================================
            // initalize the counter variables
            // =============================================================================
            seconds_passed = time::systime() + Duration::seconds(1);
            packets_received += 1;
            bytes_received_in_interval = result_size;
            // =============================================================================
            // fix on 03.09.2025 by Johann Spenrath:
            // the recv_from function is non blocking if it gets called and no packet is in
            // the sockets it just ends and saves nothing in the buffer this lead to the error
            // seen below, to fix this just run this in a match loop
            // rec_data is of type u8 which would overflow -> cast recv_data in previous and current to u32
            // get the received payload of the packet from the buffer
            // =============================================================================

            // =============================================================================
            // read the first 4 bytes of the packet payload which contain
            // the number of the nth packet, which has been sent by the client
            // =============================================================================
            previous_packet_number = u32::from_be_bytes([recv_data[0], recv_data[1], recv_data[2], recv_data[3]]);
            break;
        }
    }

    // =============================================================================
    // receive packets
    // until an exit msg is sent
    // =============================================================================
    loop {
        let result_size = TcpStream::read(&socket, &mut buf).map_err(|errno| match errno {
            network::NetworkError::Unknown(Errno::ECONNRESET) => exit = true,
            NetworkError::DeviceBusy => todo!(),
            NetworkError::InvalidAddress => todo!(),
            NetworkError::Unknown(_errno) => todo!(),
        });
        if exit {
            break;
        }
        let recv_data = &buf[..result_size.unwrap()];
        // exit condition
        if recv_data.len() >= 4 {
            // count number of packets received
            packets_received += 1;
            // =============================================================================
            // read the first 4 bytes of the packet payload which contain
            // the number of the nth packet, which has been sent by the client
            // =============================================================================
            // cast to u8 because of overflow error thrown by compiler
            current_packet_number = u32::from_be_bytes([recv_data[0], recv_data[1], recv_data[2], recv_data[3]]);

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
            bytes_received_in_interval += result_size.unwrap();

            // =============================================================================
            // if a second has passed, print out the number of bytes which
            // have been received in the current second
            // =============================================================================

            if seconds_passed < time::systime() {
                println!(
                    "[{} - {}] : [{} KB/s]",
                    interval_counter,
                    interval_counter + 1,
                    bytes_received_in_interval as f64 / 1000.0
                );
                interval_counter += 1;
                bytes_received += bytes_received_in_interval;
                bytes_received_in_interval = 0;
                seconds_passed += TimeDelta::seconds(1);
            }
        }
    }
    bytes_received += bytes_received_in_interval;

    println!(
        "[{} - {}] : [{} KB/s]",
        interval_counter,
        interval_counter + 1,
        bytes_received_in_interval as f64 / 1000.0
    );
    println!("");
    println!("==> [Received exit message from Client: End of reception!]");
    println!("");
    println!("[======================================================]");
    println!("  [Packet Payload length]      ==> {}", packet_payload_length);
    println!("  [Number of packets received] ==> {}", packets_received);
    println!("  [Total bytes received]       ==> {} B", bytes_received as f64);
    println!("  [Kbytes received]            ==> {} KB/s", bytes_received as f64 / 1000.0);
    println!("  [Average bytes received]     ==> {} B/s", (bytes_received / (interval_counter + 1)) as f64);
    println!(
        "  [Average Kbytes received]    ==> {} KB/s",
        (bytes_received / (interval_counter + 1)) as f64 / 1000.0
    );
    println!("  [packets out of order]       ==> {} / {}", packets_out_of_order, packets_received);
    println!("  [duplicated packets]         ==> {}", duplicated_packets);
    println!("[======================================================]");
    return Ok(());
}

pub fn run_tcp_client(socket: TcpStream, time_interval: TimeDelta, packet_length: u16) -> Result<()> {
    // =============================================================================
    // define variables
    // =============================================================================
    // save how many bytes have been sent in one second
    let mut bytes_sent_in_interval: u128 = 0;
    let mut interval_counter: usize = 0;
    // save the number of packets send in the time period
    let mut packets_send: u128 = 0;
    let mut send_actual: u128 = 0;
    let mut seconds_passed = TimeDelta::zero();

    println!("TCP Client starting...");

    // ======================================
    // create the packet
    // ======================================
    //let mut buf = vec![0; packet_length as usize];
    let mut buf = vec![0; packet_length as usize];
    let msg = b"I hope this works.";

    let mut packet = UdpPacket::new_unchecked(&mut buf);
    //packet.set_len(packet_length);

    // define the time for exit
    let test_finish_time = time::systime() + time_interval;
    // define counter variable seconds_passed for each passing second
    seconds_passed = time::systime() + chrono::TimeDelta::seconds(1);
    println!(
        "[  Start Time: {}  ] =========> [  End Time: {}  ]  ",
        time::systime().as_seconds_f32(),
        test_finish_time.as_seconds_f32()
    );

    // loop until the time interval has been reached
    while time::systime() < test_finish_time {
        // count each packet being send
        packets_send += 1;
        // ======================================
        // save the current number in the
        // current packet this is later being
        // read by the server for verifying
        // duplicated and out of order packets
        // ======================================
        //buf[0] = ((packets_send >> 24) & 0xff) as u8;
        //buf[1] = ((packets_send >> 16) & 0xff) as u8;
        //buf[2] = ((packets_send >> 8) & 0xff) as u8;
        //buf[3] = (packets_send & 0xff) as u8;

        // ======================================
        // send the packet
        // ======================================
        match TcpStream::write(&socket, msg) {
            Ok(_) => {
                send_actual += 1;

                // ======================================
                // count bytes send in each second
                // by adding the packet length
                // of each sent packet
                // ======================================
                bytes_sent_in_interval += packet_length as u128;
            }
            Err(_e) => {}
        }
        // ======================================
        // if a second passes print out the
        // current stats in the terminal
        // ======================================
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
            seconds_passed += TimeDelta::seconds(1);
        }
    }

    // ======================================
    // after the end of the time_inverval send
    // an exit message to the server
    // to signal end of transmission
    // ======================================

    // ======================================
    // get the total number of bytes send in
    // the defined time interval
    // ======================================
    let sent_bytes = packet_length as u128 * send_actual;
    // ======================================
    // print the end result to
    // the terminal screen
    // ======================================
    println!("==> [ reached finish time! ]");
    println!("");
    println!("[====================================================]");
    println!("  [Packet payload length]          ==> {}", packet_length);
    println!("  [Number of transmitted packets]  ==> {}", send_actual);
    println!("  [total Bytes transmitted]        ==> {} Bytes", sent_bytes);
    println!("  [total Kbytes transmitted]       ==> {} Bytes", sent_bytes as f64 / 1000.0);
    println!(
        "  [Average B/s]                   ==> {} B/s",
        (sent_bytes as f32 / time_interval.as_seconds_f32())
    );
    println!(
        "  [Average KB/s]                   ==> {} KB/s",
        (sent_bytes as f32 / time_interval.as_seconds_f32()) / 1000.0
    );
    println!("[====================================================]");
    return Ok(());
}

// =============================================================================
// function run_udp_client
// =============================================================================
// Sends "Init\n" to server, waits for an "Init\n" response,
// calls send_traffic function, then returns.
// =============================================================================

pub fn run_udp_client(socket: UdpSocket, dest_addr: SocketAddr, time_interval: TimeDelta, packet_length: u16) -> Result<()> {
    // =============================================================================
    // define variables
    // =============================================================================

    // define buffer size
    let mut buf = [0u8; 2048];

    // define Init message for connection
    let init_msg = b"Init\n";

    // =============================================================================
    // Step 1: send the init message to the receiving host
    // =============================================================================
    println!("[UDP: sending Init to {}.]", dest_addr);
    UdpSocket::send_to(&socket, init_msg, dest_addr).expect("[failed to send over UDP]");

    // =============================================================================
    // Step 2: Poll for response
    // =============================================================================
    let deadline = time::systime() + TimeDelta::seconds(5); // 5s timeout
    //let deadline = time::systime().as_seconds_f64() + 5.0; // 5s timeout
    println!("[Waiting for server reply...]");
    // ======================================
    // loop until a socket has been received
    // or the timeout is reached
    // ======================================
    loop {
        if time::systime() > deadline {
            println!("[Timeout waiting for init response.]");
            return Err(smoltcp::wire::Error);
        }
        // get the length of the reply
        let len = UdpSocket::recv_from(&socket, &mut buf).expect("[failed to receive over UDP!]").0;

        // ======================================
        // if a reply has been received and is not
        // empty convert to string
        // ======================================
        if len > 0 {
            let ack = str::from_utf8(&buf[0..len]).expect("[failed to parse received string!]");
            println!("[UDP: received ACK {:?}]", ack);
            // ======================================
            // if the ACK equals the init message
            // go to the next step and
            // send packets in a defined interval
            // ======================================
            if ack == str::from_utf8(init_msg).unwrap() {
                println!("[Received expected Init response.]");
                return udp_send_traffic(socket, dest_addr, time_interval, packet_length);
            } else {
                println!("[Unexpected data: {:?}.]", ack);
                return Err(smoltcp::wire::Error);
            }
        }
    }
}

// =============================================================================
// function udp_send_traffic:
// =============================================================================
// sender: fire N packets to 10.0.2.2:12345 and handle backpressure
// initated by run_udp_client
// sends a burst of packets in the defined time interval
// =============================================================================
pub fn udp_send_traffic(socket: UdpSocket, dest_addr: SocketAddr, time_interval: TimeDelta, packet_length: u16) -> Result<()> {
    // =============================================================================
    // define variables
    // =============================================================================
    // save how many bytes have been sent in one second
    let mut bytes_sent_in_interval: u128 = 0;
    let mut interval_counter: usize = 0;
    // save the number of packets send in the time period
    let mut packets_send: u128 = 0;
    let mut send_actual: u128 = 0;
    let mut seconds_passed = TimeDelta::zero();
    let end_msg = b"exit\n";

    // ======================================
    // create the packet
    // ======================================
    //let mut buf = vec![0; packet_length as usize];
    let mut buf = vec![0; packet_length as usize];

    let mut packet = UdpPacket::new_unchecked(&mut buf);
    packet.set_len(packet_length);

    // define the time for exit
    let test_finish_time = time::systime() + time_interval;
    // define counter variable seconds_passed for each passing second
    seconds_passed = time::systime() + chrono::TimeDelta::seconds(1);
    println!(
        "[  Start Time: {}  ] =========> [  End Time: {}  ]  ",
        time::systime().as_seconds_f32(),
        test_finish_time.as_seconds_f32()
    );

    // loop until the time interval has been reached
    while time::systime() < test_finish_time {
        // count each packet being send
        packets_send += 1;
        // ======================================
        // save the current number in the
        // current packet this is later being
        // read by the server for verifying
        // duplicated and out of order packets
        // ======================================
        buf[0] = ((packets_send >> 24) & 0xff) as u8;
        buf[1] = ((packets_send >> 16) & 0xff) as u8;
        buf[2] = ((packets_send >> 8) & 0xff) as u8;
        buf[3] = (packets_send & 0xff) as u8;

        // ======================================
        // send the packet
        // ======================================
        //UdpSocket::send_to(&socket, &buf, dest_addr).expect("failed to send over UDP");
        //loop {
        match UdpSocket::send_to(&socket, &buf, dest_addr) {
            Ok(_) => {
                //break,
                send_actual += 1;

                // ======================================
                // count bytes send in each second
                // by adding the packet length
                // of each sent packet
                // ======================================
                bytes_sent_in_interval += packet_length as u128;
            }
            Err(_e) => {
                //println!("Send busy, sleeping briefly...");
                //scheduler().sleep(1);
                //thread::sleep();
            }
        }
        //}
        /*let mut backoff = TimeDelta::microseconds(50);
        let max_backoff = TimeDelta::milliseconds(5);

        loop {
            match socket.send_to(&buf, dest_addr) {
                Ok(n) => {
                    if n != buf.len() {
                        // Handle partial send if it ever happens
                        // (for UDP this is uncommon; consider logging)
                    }
                    // reset backoff after a success
                    backoff = TimeDelta::microseconds(50);
                    break;
                }
                Err(NetworkError::DeviceBusy) => {
                    thread::sleep(backoff.num_milliseconds() as usize);
                    backoff = (backoff * 2).min(max_backoff);
                    continue;
                }
                Err(_e) => {
                    // Real error, log and decide to drop/abort
                    println!("[send_to error]");
                    break;
                }
            }
        }*/

        // ======================================
        // if a second passes print out the
        // current stats in the terminal
        // ======================================
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
            seconds_passed += TimeDelta::seconds(1);
        }
        //TODO:
        //scheduler().sleep(0.001); // e.g., sleep 1 ms, adjust accordingly
        //thread::sleep(1);
    }

    // ======================================
    // after the end of the time_inverval send
    // an exit message to the server
    // to signal end of transmission
    // ======================================

    let mut backoff = TimeDelta::microseconds(50);
    let max_backoff = TimeDelta::milliseconds(5);
    loop {
        match socket.send_to(end_msg, dest_addr) {
            Ok(n) => {
                if n != end_msg.len() {
                    println!("[partial send]")
                    // Handle partial send if it ever happens
                    // (for UDP this is uncommon; consider logging)
                }
                // reset backoff after a success
                backoff = TimeDelta::microseconds(50);
                break;
            }
            Err(NetworkError::DeviceBusy) => {
                thread::sleep(backoff.num_milliseconds() as usize);
                backoff = (backoff * 2).min(max_backoff);
                continue;
            }
            Err(_e) => {
                // Real error, log and decide to drop/abort
                println!("[send_to error]");
                break; // or return/continue based on your policy
            }
        }
    }

    // ======================================
    // get the total number of bytes send in
    // the defined time interval
    // ======================================
    let sent_bytes = packet_length as u128 * send_actual;
    // ======================================
    // print the end result to
    // the terminal screen
    // ======================================
    println!("==> [ reached finish time! ]");
    println!("");
    println!("[====================================================]");
    println!("  [Packet payload length]          ==> {}", packet_length);
    println!("  [Number of transmitted packets]  ==> {}", send_actual);
    println!("  [total Bytes transmitted]        ==> {} Bytes", sent_bytes);
    println!("  [total Kbytes transmitted]       ==> {} Bytes", sent_bytes as f64 / 1000.0);
    println!(
        "  [Average B/s]                   ==> {} B/s",
        (sent_bytes as f32 / time_interval.as_seconds_f32())
    );
    println!(
        "  [Average KB/s]                   ==> {} KB/s",
        (sent_bytes as f32 / time_interval.as_seconds_f32()) / 1000.0
    );
    println!("[====================================================]");
    return Ok(());

    //let u = UdpSocket::send_to(&sock, end_datagram, dest_addr).expect("failed to send end msg");
    //loop {
    //    match UdpSocket::send_to(&socket, &end_datagram, dest_addr) {
    //        Ok(_) => break,
    //        Err(e) => {
    //            println!("Send busy, sleeping briefly...");
    //            //scheduler().sleep(1);
    //        }
    //    }
    //}
}

// =============================================================================
// function run_udp_server:
// =============================================================================
// waits for an incoming connection request,
// if a request, is made, a reply is sent back
// and the receive_traffic function is initated
// =============================================================================
pub fn run_udp_server(socket: UdpSocket) -> Result<()> {
    // =============================================================================
    // define variables
    // =============================================================================
    let init_msg = b"Init\n";
    let mut buf = vec![0; 2048];

    // wait for a connection request from the client
    println!("[Server starting up...]");
    println!("[waiting for Init request...]");

    // =============================================================================
    // loop until an init request by a client
    // has been made
    // =============================================================================
    loop {
        let (len, sender) = UdpSocket::recv_from(&socket, &mut buf).expect("[failed to receive over UDP!]");

        // ==============================================
        // if the request contains something,
        // convert to string and check if
        // the init message is correct
        // if yes, acknowledge the request by sending
        // the init message back and
        // start the udp_receive_traffic function
        // ==============================================
        if len > 0 {
            let request_msg = str::from_utf8(&buf[0..len]).expect("[failed to parse received string!]");
            println!("[UDP: received {:?} from {:?}]", sender.ip(), sender.port());
            if request_msg == str::from_utf8(init_msg).unwrap() {
                println!("[received expected Init response.]");
                UdpSocket::send_to(&socket, init_msg, sender).expect("[failed to send init message!]");
                udp_receive_traffic(socket);
                return Ok(());
            } else {
                println!("[Unexpected data: {:?}.]", request_msg);
                panic!("[Wrong Init response!]");
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
pub fn udp_receive_traffic(sock: UdpSocket) -> Result<()> {
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
    let mut seconds_passed = TimeDelta::zero();
    // use this variable to print out the received payload length in the
    // end results
    let mut packet_payload_length = 0;
    // define exit_msg
    let exit_msg = b"exit\n";
    // define a buffer in which the received packetgets saved to
    let mut buf = vec![0; 2048];

    println!("[nettest: server listening! Send 'exit' to leave.]");

    //this lead to an error in which no packet was received now it works, when just passing the socket
    // from the calling method
    //let source_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 15));
    //let dest_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 2));
    //let sock = network::open_udp();

    // =============================================================================
    // wait for the reception of the first packet
    // then start the timer and the loop
    // =============================================================================
    let deadline = time::systime() + Duration::seconds(5); // set 5s deadline for packet to arrive
    println!("[waiting for first packet...]");
    loop {
        if time::systime() > deadline {
            panic!("[timeout waiting for first packet]");
        }

        let result = UdpSocket::recv_from(&sock, &mut buf).expect("failed to receive datagram");
        let recv_data = &buf[..result.0];
        if recv_data.len() >= 4 {
            println!("[UDP: received first packet from {:?}:{:?}]", result.1.ip(), result.1.port());
            println!("");
            println!("[======================================================]");
            println!("  [  Start Time: {}  ]", time::systime().as_seconds_f64());
            println!("[======================================================]");
            println!("");
            packet_payload_length = result.0;
            // =============================================================================
            // initalize the counter variables
            // =============================================================================
            seconds_passed = time::systime() + Duration::seconds(1);
            packets_received += 1;
            bytes_received_in_interval = result.0;
            // =============================================================================
            // fix on 03.09.2025 by Johann Spenrath:
            // the recv_from function is non blocking if it gets called and no packet is in
            // the sockets it just ends and saves nothing in the buffer this lead to the error
            // seen below, to fix this just run this in a match loop
            // rec_data is of type u8 which would overflow -> cast recv_data in previous and current to u32
            // get the received payload of the packet from the buffer
            // =============================================================================

            // =============================================================================
            // read the first 4 bytes of the packet payload which contain
            // the number of the nth packet, which has been sent by the client
            // =============================================================================
            previous_packet_number = u32::from_be_bytes([recv_data[0], recv_data[1], recv_data[2], recv_data[3]]);
            break;
        }
    }

    // =============================================================================
    // receive packets
    // until an exit msg is sent
    // =============================================================================
    loop {
        let result = UdpSocket::recv_from(&sock, &mut buf).expect("failed to parse datagram");
        let recv_data = &buf[..result.0];
        // exit condition
        if recv_data == exit_msg {
            break;
        }
        if recv_data.len() >= 4 {
            // count number of packets received
            packets_received += 1;
            // =============================================================================
            // read the first 4 bytes of the packet payload which contain
            // the number of the nth packet, which has been sent by the client
            // =============================================================================
            // cast to u8 because of overflow error thrown by compiler
            current_packet_number = u32::from_be_bytes([recv_data[0], recv_data[1], recv_data[2], recv_data[3]]);

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
            bytes_received_in_interval += result.0;

            // =============================================================================
            // if a second has passed, print out the number of bytes which
            // have been received in the current second
            // =============================================================================

            if seconds_passed < time::systime() {
                println!(
                    "[{} - {}] : [{} KB/s]",
                    interval_counter,
                    interval_counter + 1,
                    bytes_received_in_interval as f64 / 1000.0
                );
                interval_counter += 1;
                bytes_received += bytes_received_in_interval;
                bytes_received_in_interval = 0;
                seconds_passed += TimeDelta::seconds(1);
            }
        }
    }
    bytes_received += bytes_received_in_interval;

    println!(
        "[{} - {}] : [{} KB/s]",
        interval_counter,
        interval_counter + 1,
        bytes_received_in_interval as f64 / 1000.0
    );
    println!("");
    println!("==> [Received exit message from Client: End of reception!]");
    println!("");
    println!("[======================================================]");
    println!("  [Packet Payload length]      ==> {}", packet_payload_length);
    println!("  [Number of packets received] ==> {}", packets_received);
    println!("  [Total bytes received]       ==> {} B", bytes_received as f64);
    println!("  [Kbytes received]            ==> {} KB/s", bytes_received as f64 / 1000.0);
    println!("  [Average bytes received]     ==> {} B/s", (bytes_received / (interval_counter + 1)) as f64);
    println!(
        "  [Average Kbytes received]    ==> {} KB/s",
        (bytes_received / (interval_counter + 1)) as f64 / 1000.0
    );
    println!("  [packets out of order]       ==> {} / {}", packets_out_of_order, packets_received);
    println!("  [duplicated packets]         ==> {}", duplicated_packets);
    println!("[======================================================]");
    return Ok(());
}
