// =============================================================================
// FILE        : benchmark.rs
// AUTHOR      : Johann Spenrath <johann.spenrath@hhu.de>
// DESCRIPTION : functions for sending and receiving packets and printing stats
// =============================================================================
// NOTES:
// =============================================================================
// DEPENDENCIES:
// =============================================================================
use crate::scheduler;
use crate::{network, timer};
use alloc::vec;
use log::{error, info, warn};
use smoltcp::iface::SocketHandle;
use smoltcp::socket::udp::RecvError;
use smoltcp::socket::udp::SendError;
use smoltcp::wire::Ipv4Address;
use smoltcp::wire::{IpAddress, IpEndpoint};

// enable/disable additional poll after each send/receive operation
const ENABLE_POLL: bool = true;

// =============================================================================
// function benchmark
// =============================================================================
// start client or server from this function
// =============================================================================

pub fn benchmark(receive: bool) {
    let dest_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 2));
    let dest_port: u16 = 2000;
    let source_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 15));
    let source_port = 1798;
    let timing_interval = 20;
    let packet_length: u16 = 64;

    let dest_addr = (dest_ip, dest_port);

    let sock = network::open_udp();
    let _ = network::bind_udp(sock, source_ip, source_port).expect("failed to bind udp socket!");

    if receive {
        return run_udp_server(sock, dest_addr).expect("failed to start server!");
    }
    return run_udp_client(sock, dest_addr, timing_interval, packet_length).expect("failed to start client");
}
// =============================================================================
// function run_udp_client
// =============================================================================
// Sends "Init\n" to server, waits for an "Init\n" response, then returns.
// =============================================================================

pub fn run_udp_client(sock: SocketHandle, addr: (IpAddress, u16), timing_interval: u16, packet_length: u16) -> Result<(), &'static str> {
    // =============================================================================
    // define variables
    // =============================================================================
    let mut buf = [0u8; 512];
    // create the endpoint which the client wants to send to
    let endpoint = IpEndpoint::new(addr.0, addr.1);

    // =============================================================================
    // Step 1: send the init message to the receiving host
    // =============================================================================
    info!("UDP: sending Init to {}.", endpoint);
    let _ = network::send_datagram(sock, addr.0, addr.1, b"Init\n").map_err(|_| "send Init failed")?;

    // =============================================================================
    // Step 2: Poll for response
    // =============================================================================
    let deadline = crate::timer().systime_ms() + 5000; // 5s timeout
    info!("Waiting for server reply...");
    // ======================================
    // Step 3:
    // loop until a socket has been received
    // or the timeout is reached
    // ======================================
    loop {
        if crate::timer().systime_ms() > deadline {
            network::close_socket(sock);
            info!("timeout waiting for Init response");
            return Err("timeout waiting for Init response");
        }

        if let Ok((size, meta)) = network::receive_datagram(sock, &mut buf) {
            let recv_data = &buf[..size];
            info!("UDP: received from {}: {:?}", meta.endpoint, recv_data);
            // ======================================
            // if the ACK equals the init message
            // go to the next step and
            // send packets in a defined interval
            // ======================================
            if recv_data == b"Init\n" {
                info!("Received expected Init response");
                let _ = udp_send_traffic(sock, addr, timing_interval, packet_length).expect("failed to start send loop!");
                return Ok(());
            } else {
                warn!("Unexpected data: {:?}", recv_data);
                return Err("Wrong Init response");
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
// old test worked until the TX ring filled, then it paniced the kernel because call .expect("Failed to send UDP datagram").
// new version doesn’t crash because it handles backpressure (BufferFull) by polling/yielding and retrying instead of panicking.
// =============================================================================

pub fn udp_send_traffic(sock: SocketHandle, addr: (IpAddress, u16), interval: u16, packet_length: u16) -> Result<(), &'static str> {
    // =============================================================================
    // define variables
    // =============================================================================
    let mut bytes_sent_in_interval = 0;
    let mut interval_counter: usize = 0;
    let mut packet_number: u32 = 0;
    let mut buf = vec![0; packet_length as usize];
    let datagram: &mut [u8] = &mut buf;
    let mut seconds_passed = 0;

    // define the time for exit
    let test_finish_time = timer().systime_ms() + 20_000;
    // define counter variable seconds_passed for each passing second
    seconds_passed = timer().systime_ms() + 1_000;
    info!("Start: {} - End: {}", timer().systime_ms(), test_finish_time);
    info!("--------------------------------------------------------");

    // loop until the time interval has been reached
    while timer().systime_ms() < test_finish_time {
        // ======================================
        // send the packet
        // ======================================
        // retry until queued; the poll thread
        // will drain TX between retries
        //loop {
        // ======================================
        match network::send_datagram(sock, addr.0, addr.1, datagram) {
            Ok(()) => {
                //break;
                // count each packet being send
                packet_number += 1;
                // ======================================
                // save the current number in the
                // current packet this is later being
                // read by the server for verifying
                // duplicated and out of order packets
                // ======================================
                datagram[0] = ((packet_number >> 24) & 0xff) as u8;
                datagram[1] = ((packet_number >> 16) & 0xff) as u8;
                datagram[2] = ((packet_number >> 8) & 0xff) as u8;
                datagram[3] = (packet_number & 0xff) as u8;

                // ======================================
                // count bytes send in each second
                // by adding the packet length
                // of each sent packet
                // ======================================
                bytes_sent_in_interval += packet_length;
            }
            Err(SendError::BufferFull) => {
                info!("Buffer full");
                // give the poll method time to flush and to finish ARP, then retry
                //scheduler().sleep(1);
            }
            Err(e) => panic!("(UDP Send Test) send failed: {e:?}"),
        }
        //}

        // ======================================
        // if a second passes print out the
        // current stats in the terminal
        // ======================================
        if seconds_passed < timer().systime_ms() {
            info!(
                "{} - {} : {} KB/s",
                interval_counter,
                interval_counter + 1,
                (bytes_sent_in_interval as f64) / 1000.0
            );
            // update counters after a passed second
            interval_counter += 1;
            bytes_sent_in_interval = 0;
            seconds_passed += 1_000;
        }
        // Let other threads run / allow network stack to poll
        if ENABLE_POLL {
            network::poll_sockets;
            //scheduler().sleep(20);
        }
    }

    // ======================================
    // after the end of the inverval send
    // an exit message to the server
    // to signal end of transmission
    // ======================================
    let end_datagram: &[u8] = b"exit\n";
    match network::send_datagram(sock, addr.0, addr.1, end_datagram) {
        Ok(()) => {}
        Err(SendError::BufferFull) => {
            info!("Buffer full");
        }
        Err(e) => panic!("(UDP Send Test) send failed: {e:?}"),
    }

    // ======================================
    // get the total number of bytes send in
    // the defined time interval
    // ======================================
    let sent_bytes = packet_length as u32 * packet_number;
    info!("--------------------------------------------------------");
    info!("Packet payload length: {}", packet_length);
    info!("Packets transmitted : {}", packet_number);
    info!("Bytes transmitted: {}", sent_bytes);
    info!("Average: {} KB/s", (sent_bytes as f64 / interval as f64) / 1000.0);
    return Ok(());
}

pub fn run_udp_server(sock: SocketHandle, addr: (IpAddress, u16)) -> Result<(), &'static str> {
    // =============================================================================
    // define variables
    // =============================================================================
    let init_msg = b"Init\n";
    let mut buf = vec![0; 2048];
    let _deadline = crate::timer().systime_ms() + 20_000; // 20s timeout

    // wait for a connection request from the client
    // enable this if needed
    info!("Server starting up...");
    info!("waiting for Init response.");

    loop {
        //if crate::timer().systime_ms() > deadline {
        //    network::close_socket(sock);
        //    info!("timeout waiting for Init response");
        //    return Err("timeout waiting for Init response");
        //}

        // ==============================================
        // acknowledge the request by sending
        // the init message back and
        // start the udp_receive_traffic function
        // ==============================================

        if let Ok((size, meta)) = network::receive_datagram(sock, &mut buf) {
            let recv_data = &buf[..size];
            info!("UDP: received from {}: {:?}", meta.endpoint, recv_data);

            if recv_data == init_msg {
                info!("Received expected Init response");
                let endpoint = IpEndpoint::new(addr.0, addr.1);
                info!("UDP: sending Init to {}", endpoint);
                let _ = network::send_datagram(sock, addr.0, addr.1, recv_data).expect("failed to send Init ACK!");
                let _ = udp_receive_traffic(sock).expect("failed to start receive loop!");
                return Ok(());
            } else {
                warn!("Unexpected data: {:?}", recv_data);
                return Err("Unexpected data");
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
pub fn udp_receive_traffic(sock: SocketHandle) -> Result<(), &'static str> {
    // define variables
    let mut packets_received: u32 = 0;
    let mut packets_out_of_order: u32 = 0;
    let mut duplicated_packets: u32 = 0;
    #[allow(unused_variables)]
    let mut current_packet_number: u32 = 0;
    let mut previous_packet_number: u32 = 0;
    let mut bytes_received_in_interval: usize = 0;
    let mut seconds_passed = 0;
    // use this variable to print out the packet length at the
    // end in the results
    let mut info_payload_length = 0;
    let mut interval_counter: usize = 0;
    let mut bytes_received: usize = 0;
    // define exit_msg
    let exit_msg = b"exit\n";
    // define a buffer in which the received packetgets saved to
    let mut buf = [0u8; 2048];

    //this lead to an error in which no packet was received now it works, when just passing the socket
    // from the calling method
    //let source_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 15));
    //let dest_ip = smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 2));
    //let listening_port = 1798;
    //let sending_port = 12345;
    //let sock = network::open_udp();

    // =============================================================================
    // wait for the reception of the first packet
    // then start the timer and the loop
    // =============================================================================

    // enable this if needed
    let _deadline = timer().systime_ms() + 5_000; // set 5s deadline for packet to arrive
    info!("waiting for first packet ");
    loop {
        //if crate::timer().systime_ms() > deadline {
        //network::close_socket(sock);
        //return Err("timeout waiting for first packet ");
        //("timeout waiting for first packet");
        //}

        if let Ok((size, meta)) = network::receive_datagram(sock, &mut buf) {
            // rec_data is of type u8 which would overflow -> cast recv_data in previous and current to u32
            let recv_data = &buf[..size];
            info_payload_length = size;
            //info!("UDP: received first packet from {}: {:?}", meta.endpoint, recv_data);
            info!("UDP: received first packet from {}.", meta.endpoint);
            info!("Start: {}", timer().systime_ms());
            // =============================================================================
            // initalize the counter variables
            // =============================================================================
            seconds_passed = timer().systime_ms() + 1_000;
            packets_received += 1;
            bytes_received_in_interval = size;
            // =============================================================================
            // read the first 4 bytes of the packet payload which contain
            // the number of the nth packet, which has been sent by the client
            // =============================================================================
            previous_packet_number = u32::from_be_bytes([recv_data[0], recv_data[1], recv_data[2], recv_data[3]]);
            break;
        }
        // use this for prohibiting the queue from filling up
    }

    // =============================================================================
    // receive packets
    // until an exit msg is sent
    // =============================================================================
    loop {
        //let size = network::receive_datagram(sock, &mut buf).unwrap().0;
        match network::receive_datagram(sock, &mut buf) {
            Ok((size, _meta)) => {
                let recv_data = &buf[..size];
                if recv_data == exit_msg {
                    break;
                }

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
                bytes_received_in_interval += size;
            }
            Err(RecvError::Exhausted) => {
                // Nothing ready yet — just wait and retry
                scheduler().sleep(1); // or network::poll() if appropriate
                continue;
            }
            Err(e) => {
                error!("Receive failed: {:?}", e);
                scheduler().sleep(1);
                continue;
            }
        }

        // =============================================================================
        // if a second has passed, print out the number of bytes which
        // have been received in the current second
        // =============================================================================
        if seconds_passed < timer().systime_ms() {
            info!(
                "{} - {}: {} KB/s",
                interval_counter,
                interval_counter + 1,
                bytes_received_in_interval as f64 / 1000.0
            );
            interval_counter += 1;
            bytes_received += bytes_received_in_interval;
            bytes_received_in_interval = 0;
            seconds_passed += 1_000;
        }
        // disable/enable
        // poll sockets after every packet thats being received
        if ENABLE_POLL {
            network::poll_sockets();
        }
    }
    bytes_received += bytes_received_in_interval;

    info!("{} - {}: {} KB/s", interval_counter, interval_counter + 1, bytes_received_in_interval / 1000);
    info!("Received exit: End reception");
    info!("--------------------------------------------------------------");
    info!("Packet payload length {}", info_payload_length);
    info!("Number of packets received : {}", packets_received);
    info!("Bytes received : {} KB/s", bytes_received as f64 / 1000.0);
    info!("Average Bytes received : {} KB/s", (bytes_received / (interval_counter + 1)) as f64 / 1000.0);
    info!("packets out of order: {}", packets_out_of_order / packets_received);
    info!("duplicated packets: {}", duplicated_packets);
    info!("--------------------------------------------------------------");
    return Ok(());
}
