#!/usr/bin/env python3

import argparse
import os
import socket
import time
from datetime import datetime, timedelta

## =============================================================================
## FILE        : server.py
## AUTHOR      : Johann Spenrath <johann.spenrath@hhu.de>
## DESCRIPTION : functions for sending and receiving packets and printing stats
## =============================================================================
## NOTES:
## =============================================================================
## DEPENDENCIES:
## =============================================================================

BUFFER_SIZE = 40960

def receive_traffic(sock): 

    packet_count = 0
    packets_received = 0
    packets_out_of_order = 0
    duplicated_packets = 0

    current_packet_number = 0
    previous_packet_number = 0
    interval_counter = 0

    bytes_received = 0
    bytes_received_in_interval = 0
    bytes_received_total = 0
    exit_msg = b"exit\n"


    # Create a UDP socket
    #socket_handle = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    # Bind the socket to the port
    #server_address = (ip, port)
    #socket_handle.bind(server_address)
    #print(f"## server is listening from {ip} on Port {port} ")

    # =====================================================================
    # Step 1: get the first packet
    # then start the loop
    # =====================================================================
    payload, address = sock.recvfrom(BUFFER_SIZE) 
    if not payload:
        return 


    # =================================
    # Display the current time at start
    # =================================
    print(f"start: {datetime.now().time()}")

    seconds_passed = int(time.time()) + 1

    # =================================
    # set packets_received +1 for the 
    # first received packet
    # save the first 4 byte in 
    # previous_packet_number
    # =================================
    packets_received += 1

    previous_packet_number = (payload[0] << 24) + (payload[1] << 16) + (payload[2] << 8) + payload[3]
    bytes_received_in_interval = len(payload)

    # print result to log file
    today = datetime.today()
    current_date = datetime(today.year, today.month, today.day, today.hour, today.minute, today.second)
    f = open(f"./results/nettest_benchmark_{current_date}.txt", "a")
    # =====================================================================
    # Receive packets until an exit msg is send
    # =====================================================================
    while True:
        #try:
        data, _ = sock.recvfrom(BUFFER_SIZE)
        #except socket.timeout:
        #    print("No more packets received: timing out.")
        #    break
        # =================================
        # extract data payload and address 
        # from where the packet was sent
        # if msg = exit: end the receive loop
        # =================================
        #data, _ = sock.recvfrom(BUFFER_SIZE) 
        if data == exit_msg:
            break

        received_message = data

        # =================================
        # count the number of packets received
        # in each iteration
        # =================================
        packets_received += 1
        current_packet_number = (received_message[0] << 24) + (received_message[1] << 16) + (received_message[2] << 8) + received_message[3];

        #packet_bytes = received_message[:4]
        #packet_number = int.from_bytes(packet_bytes, byteorder='big')
        #print(f"Parsed packet number: {packet_number}")

        # =================================
        # compare previous with current packet
        # if packet numbers equal then, the 
        # packet has been retransmitted
        # =================================
        if current_packet_number == previous_packet_number:
            duplicated_packets += 1
        
        # =================================
        # check if the current packet 
        # has arrived in the wrong order
        # =================================
        elif current_packet_number != (previous_packet_number+1) or current_packet_number < previous_packet_number:
            packets_out_of_order += 1


        # =================================
        # update the packet
        # =================================
        previous_packet_number = current_packet_number
        bytes_received_in_interval = bytes_received_in_interval + len(data)


        # =================================
        # write current bytes per second
        # =================================
        if seconds_passed < int(time.time()):
            string = f"{interval_counter}-{interval_counter + 1}: {bytes_received_in_interval / 1000.0} KB/s"
            print(string)
            f.write(string + "\n")


            interval_counter += 1
            bytes_received = bytes_received + bytes_received_in_interval
            # reset bytes received 
            bytes_received_in_interval = 0
            # set to next second passed
            seconds_passed +=1
    
    bytes_received = bytes_received + bytes_received_in_interval
    average_bytes = (bytes_received / (interval_counter + 1)) / 1000

    print(f"{interval_counter} - {interval_counter + 1}: {bytes_received_in_interval/1000.0} KB/s")
    print(f"Received exit: End reception")
    print(f"------------------------------------------------------------------------")
    print(f"Number of packets received : {packets_received}")
    print(f"Bytes received             : {bytes_received / 1000} KB")
    print(f"Average Bytes received     : {average_bytes} KB/s")
    print(f"packets out of order       : {packets_out_of_order} / {packets_received}")
    print(f"duplicated packets         : {duplicated_packets}")
    print(f"------------------------------------------------------------------------")
    f.write(f"{interval_counter} - {interval_counter + 1}: {bytes_received_in_interval/1000.0} KB/s \n")
    f.write(f"Received exit: End reception\n")
    f.write(f"------------------------------------------------------------------------")
    f.write(f"Number of packets received : {packets_received}\n")
    f.write(f"Bytes received             : {bytes_received / 1000} KB\n")
    f.write(f"Average Bytes received     : {average_bytes} KB/s\n")
    f.write(f"packets out of order       : {packets_out_of_order} / {packets_received}\n")
    f.write(f"duplicated packets         : {duplicated_packets}\n")
    #print(f"Packet #{packet_count} from {address}: {data.decode(errors='ignore')}")

    #print(" payload size ", data.len())
    #send_data = input("Type some text to send => ")
    #s.sendto(send_data.encode('utf-8'), address)
    #print("\n\n 1. Server sent : ", send_data,"\n\n")


def server(sock, address, address_remote):

    print(f"nettest: server listening on {address}! Send 'exit' to leave.")
    print("Do Ctrl+c to exit the program !!")
    # define init message
    init_msg = b"Init\n"

    while True:
        data, address_r = sock.recvfrom(BUFFER_SIZE)
        if not data:
            continue

        if data == init_msg:
            print("received Init request.")
            sock.sendto(data, address_remote)
            print(address_remote)
            print(data)
            print(address)
            return receive_traffic(sock) 


def tcp_server(conn, addr):
    print(f"Connection from {addr}")
    try:
        while True:
            data = conn.recv(1024)
            if not data:
                # No data means the connection was closed
                break
            print(f"Received: {data.decode(errors='ignore')}")
            # Optionally, echo data back:
            conn.sendall(data)
    except Exception as e:
        print(f"Error with {addr}: {e}")
    finally:
        conn.close()
        print(f"Connection with {addr} closed.")




def send_traffic(sock, addr, packet_length, duration):

    bytes_sent_in_interval = 0
    interval_counter = 0
    packet_number = 0

    # make sure packet_length is > 4
    packet_length = max(packet_length, 4)
    packet = bytearray(packet_length)
    print(f"Packet length: {len(packet)}")  # should be 64
    pps = 90

    interval = 1.0 / pps

    #test_finish_time = int(time.time()) + interval 
    seconds_passed = int(time.time()) + 1 
    start_time = time.time()
    next_send_time = start_time
    print(f"Start: {start_time} - End: {duration}")
    print(f"-------------------------------------------------------")

    # log the results 
    today = datetime.today()
    current_date = datetime(today.year, today.month, today.day, today.hour, today.minute, today.second)
    f = open(f"./results/nettest_benchmark_{current_date}.txt", "a")
    while (time.time() - start_time) < duration:
        packet_number = packet_number + 1

        packet[0] = (packet_number >> 24) & 0xFF
        packet[1] = (packet_number >> 16) & 0xFF
        packet[2] = (packet_number >> 8) & 0xFF
        packet[3] = (packet_number ) & 0xFF

        sock.sendto(packet, addr)

        bytes_sent_in_interval = bytes_sent_in_interval + packet_length

        if seconds_passed < int(time.time()):
            string = f"{interval_counter}-{interval_counter + 1}: {bytes_sent_in_interval / 1000.0} KB/s"
            print(string)
            f.write(string + "\n")
            interval_counter += 1
            # reset bytes received 
            bytes_sent_in_interval = 0
            # set to next second passed
            seconds_passed +=1
        
        #time.sleep(0.5)
        if interval is not None:
            next_send_time += interval
            sleep_for = next_send_time - time.time()
            if sleep_for > 0:
                time.sleep(sleep_for)
            else:
                # if we're behind schedule, snap to now to avoid drift explosion
                next_send_time = time.time()

    end_msg = b"exit\n"
    sock.sendto(end_msg, addr)

    send_bytes = packet_length * packet_number

    print(f"------------------------------------------------------------------------")
    print(f"exit: End sending")
    print(f"------------------------------------------------------------------------")
    print(f"Packets transmitted : {packet_number}")
    print(f"Total bytes tranmitted: {send_bytes}")
    print(f"Average Bytes  : {(send_bytes/interval)/1000.0} KB/s")
    #print(f"Packet #{packet_count} from {address}: {data.decode(errors='ignore')}")
    print(f"------------------------------------------------------------------------")
    f.write(f"Packets transmitted : {packet_number}\n")
    f.write(f"Total bytes tranmitted: {send_bytes}\n")
    f.write(f"Average Bytes  : {(send_bytes/interval)/1000.0} KB/s\n")

        


def client(sock, addr, packet_length, interval):


    #print(f"nettest: client listening on {local_address}! Send 'exit' to leave.")
    print("Do Ctrl+c to exit the program !!")
    print("UDP: sending Init to {addr}")

    init_msg = b"Init\n"
    # send init msg to server 
    sock.sendto(init_msg, addr)


    while True:
        data, addr = sock.recvfrom(BUFFER_SIZE)
        if not data:
            continue


        if data == init_msg:
            return send_traffic(sock, addr, packet_length, interval)





def main():


    # tcp
    #server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    #server.bind(("127.0.0.1", 2000))
    #server.listen(1)
    #print(f"Listening on 127.0.0.1:2000")

    #while True:
    #    conn, addr = server.accept()
    #    print(conn)
    #    print(addr)
    #    return tcp_server(conn, addr)


    ap = argparse.ArgumentParser(description="UDP packet sender with 4-byte sequence header.")
    ap.add_argument("host", help="Server IP / hostname")
    ap.add_argument("port", type=int, help="Server port")
    ap.add_argument("host_remote", help="Server IP / hostname")
    ap.add_argument("port_remote", type=int, help="Server port")
    ap.add_argument("--mode", "-m", type=int, default=0,
                        help="specify mode, 0: Server, 1: Client" )
    group = ap.add_mutually_exclusive_group()
    group.add_argument("--count", "-c", type=int, default=9_000,
                       help="Number of packets to send. Default: 9999")
    group.add_argument("--duration", "-d", type=int, default=20,
                       help="Seconds to send for (overrides --count)")
    group.add_argument("--packet_length", "-p", type=int, default=1024,
                       help="define the packet length (client mode)")
    ap.add_argument("--pps", type=float,
                    help="Packets per second (rate limit). If unset, send as fast as possible.")
    ap.add_argument("--no-connect", action="store_true",
                    help="Use sendto() instead of connect()+send()")
    args = ap.parse_args()




    # set the arguments for the application
    ip = args.host
    port = args.port 
    ip_remote = args.host_remote
    port_remote = args.port_remote

    address = (ip, port)
    address_remote = (ip_remote, port_remote)

    # set the default timer, for how long packets should be sent
    timing_interval = args.duration

    packet_length = args.packet_length

    # create socket handle
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    #sock.settimeout(5.0)  

    # Bind the socket to the port
    sock.bind(address)

    if args.mode == 1:
        return client(sock, address_remote, packet_length, timing_interval )

    return server(sock, address, address_remote)

if __name__ == "__main__":
    main()
