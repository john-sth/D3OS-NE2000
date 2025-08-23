## =============================================================================
## FILE        : benchmark.rs
## AUTHOR      : Johann Spenrath <johann.spenrath@hhu.de>
## DESCRIPTION : functions for sending and receiving packets and printing stats
## =============================================================================
## NOTES:
## =============================================================================
## DEPENDENCIES:
## =============================================================================
import socket
import sys
from datetime import datetime, timedelta
import time


def receive_traffic(sock): 
    packet_count = 0
    #buffer size 
    buffer_size = 4096000

    packets_received = 0
    packets_out_of_order = 0
    duplicated_packets = 0
    bytes_received = 0
    current_packet_number = None
    previous_packet_number = 0
    interval_counter = 0
    bytes_received_in_interval = 0
    bytes_received_total = 0

    # Create a UDP socket
    #socket_handle = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    # Bind the socket to the port
    #server_address = (ip, port)
    #socket_handle.bind(server_address)
    #print(f"## server is listening from {ip} on Port {port} ")

    data, address = sock.recvfrom(buffer_size) 
    if not data:
        return 


    print(f"start: {datetime.now().time()}")
    seconds_passed = int(time.time()) + 1

    received_message = data

    packets_received += 1
    previousPacketNumber = (received_message[0] << 24) + (received_message[1] << 16) + (received_message[2] << 8) + received_message[3];
    bytes_received_in_interval = len(data)

    #seconds_passed = int(time.time() + 1)
    #print("\n\n 2. Server received: ", data.decode('utf-8'), "\n\n")
    while True:
        # extract data payload and address from where the packet was sent
        #try:
        data, address = sock.recvfrom(buffer_size) 
        if data.strip() == b"exit":
            break

        packets_received += 1
        current_packet_number = (received_message[0] << 24) + (received_message[1] << 16) + (received_message[2] << 8) + received_message[3];

        if current_packet_number == previous_packet_number:
            duplicated_packets += 1
        
        elif current_packet_number != (previous_packet_number+1) or current_packet_number < previous_packet_number:
            packets_out_of_order += 1

        previous_packet_number = current_packet_number
        bytes_received_in_interval = bytes_received_in_interval + len(data)

        if seconds_passed <= int(time.time()):
            print(f"{interval_counter}-{interval_counter + 1}: {bytes_received_in_interval / 1000} KB/s")
            interval_counter += 1
            bytes_received = bytes_received + bytes_received_in_interval
            bytes_received_in_interval = 0
            seconds_passed +=1
    
    bytes_received_total = bytes_received + bytes_received_in_interval

    print(f"{interval_counter} - {interval_counter + 1}: {bytes_received_in_interval/1000} KB/s")
    print(f"Received exit: End reception")
    print(f"------------------------------------------------------------------------")
    print(f"Number of packets received : {packets_received}")
    print(f"Total bytes received       : {bytes_received_total}")
    print(f"Bytes received             : {bytes_received / 1000} KB/s")
    print(f"Bytes received             : {bytes_received } B/s")
    print(f"Average Bytes received     : {(bytes_received / (interval_counter+1)) / 1000} KB/s")
    print(f"packets out of order       : {packets_out_of_order} / {packets_received}")
    print(f"duplicated packets         : {duplicated_packets}")
    #print(f"Packet #{packet_count} from {address}: {data.decode(errors='ignore')}")
    print(f"------------------------------------------------------------------------")

    #print(" payload size ", data.len())
    #send_data = input("Type some text to send => ")
    #s.sendto(send_data.encode('utf-8'), address)
    #print("\n\n 1. Server sent : ", send_data,"\n\n")

def server(sock, local_address):

    buffer_size = 409600
    # Print the local address (currently hardcoded)
    print(f"nettest: server listening on {local_address}! Send 'exit' to leave.")
    print("Do Ctrl+c to exit the program !!")

    while True:
        data, address = sock.recvfrom(buffer_size)
        if not data:
            continue

        print(data)
        print(data.strip())
        print(address)
        address = ('127.0.0.1', 1798)
        print(address)
        if data.strip() == b"Init":

            print(address)
            sock.sendto(data, address)
            return receive_traffic(sock) 



def main():

    # set the arguments for the server_address
    local_address = "127.0.0.1"
    port = 12345

    server_address = (local_address, port)

    # set the default timer, for how long packets should be sent

    timing_interval = 20

    # create socket handle
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    # Bind the socket to the port
    sock.bind(server_address)

    return server(sock, local_address)


if __name__ == "__main__":
    main()


#def kb_bar(kb, scale=200):  # 200 KB per block (tweak)
#    blocks = int(max(1, kb) / scale)
#    return "█" * min(blocks, 60)  # cap width
## get the arguments
#if len(sys.argv) == 3:
#    # Get "IP address of Server" and also the "port number" from argument 1 and argument 2
#    ip = sys.argv[1]
#    port = int(sys.argv[2])
#else:
#    print("Run like : python3 server.py <arg1:server ip:this system IP 192.168.1.6> <arg2:server port:4444 >")
#    exit(1)
#
#
#
##while True:
##    data, address = socket_handle.recvfrom(buffer_size)
##    if data.getData().decode().strip() == "Init":
##        receive_traffic()
