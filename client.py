#!/usr/bin/env python3
import argparse
import os
import socket
import time

BUFFER_SIZE = 65535

'''
def send_packets(host: str,
                 port: int,
                 payload_size: int = 1200,
                 count: int | None = 10_000,
                 pps: float | None = None,
                 duration: float | None = None,
                 connect_mode: bool = True):
    """
    Send UDP packets with a 4-byte big-endian sequence number header.
    Matches the server that expects:
      - 'Init' (handshake), echoed back
      - data packets: [seq(4 bytes)] + payload
      - 'exit' to terminate
    """

    if payload_size < 4:
        raise ValueError("payload_size must be at least 4 bytes (to hold the 4-byte sequence).")

    addr = (host, port)

    local_address = "127.0.0.1"
    port = 12345
    client_address = (local_address, port)

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    #sock.settimeout(2.0)
    sock_rec = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock_rec.bind(client_address)


    if connect_mode:
        #avoids passing addr on every send
        sock.connect(addr)
    

    # --- Handshake ---
    init = b"Init\n"
    #sock.send(init) if connect_mode else sock.sendto(init, client_address)
    sock.sendto(init, addr)
    #try:
    while True:
        data, address = sock_rec.recvfrom(BUFFER_SIZE)
        if not data:
            continue
    #data = sock.recv(1024) if connect_mode else sock.recvfrom(1024)[0]
    #except socket.timeout:
        #print("No echo from server after 'Init' (timeout). You can still send, but the server may not be ready.")
    #else:
        if data.strip() == b"Init":
            print("Handshake OK (server echoed 'Init').")
            break
        else:
            print(f"Unexpected handshake reply: {data!r}")

    # --- Rate control setup ---
    interval = None
    if pps and pps > 0:
        interval = 1.0 / pps

    start_time = time.time()
    next_send_time = start_time
    sent = 0
    bytes_sent = 0

    # Decide stop condition
    def should_stop():
        if duration is not None and (time.time() - start_time) >= duration:
            return True
        if count is not None and sent >= count:
            return True
        return False

    # --- Sending loop ---
    seq = 0
    try:
        while not should_stop():
            # Packet = 4-byte big-endian sequence + payload
            sent += 1
            #header = seq.to_bytes(4, byteorder="big")
            #payload = (os.urandom(payload_size ) if payload_size > 4 else b"")
            #payload_size = max(payload_size, 4)
            payload = bytearray(os.urandom(payload_size)) if payload_size > 4 else bytearray(4)

            payload[0] = (sent >> 24) & 0xFF
            payload[1] = (sent >> 16) & 0xFF
            payload[2] = (sent >> 8) & 0xFF
            payload[3] = (sent ) & 0xFF

            print(payload[0])
            print(payload[1])
            print(payload[2])
            print(payload[3])
            if connect_mode:
                sock.send(payload)
                #time.sleep(0.2)
            else:
                sock.sendto(payload, addr)
                #time.sleep(0.2)

            bytes_sent += len(payload)
            seq += 1

            # simple rate control (pps)
            if interval is not None:
                next_send_time += interval
                sleep_for = next_send_time - time.time()
                if sleep_for > 0:
                    time.sleep(sleep_for)
                else:
                    # if we're behind schedule, snap to now to avoid drift explosion
                    next_send_time = time.time()
    finally:
        # Tell server to stop
        stop_msg = b"exit\n"
        print(stop_msg)
        try:
            sock.send(stop_msg) if connect_mode else sock.sendto(stop_msg, addr)
        except Exception:
            pass
        sock.close()

    elapsed = max(1e-9, time.time() - start_time)
    rate_bps = bytes_sent * 8 / elapsed
    rate_kBps = bytes_sent / 1000 / elapsed
    print(f"Sent {sent} packets, {bytes_sent} bytes in {elapsed:.2f}s")
    print(f"Throughput ≈ {rate_kBps:.2f} KB/s ({rate_bps/1e6:.2f} Mb/s)")

def main():
    ap = argparse.ArgumentParser(description="UDP packet sender with 4-byte sequence header.")
    ap.add_argument("host", help="Server IP / hostname")
    ap.add_argument("port", type=int, help="Server UDP port")
    ap.add_argument("--payload-size", "-s", type=int, default=1200,
                    help="Total bytes per packet (>=4). Default: 1200")
    group = ap.add_mutually_exclusive_group()
    group.add_argument("--count", "-c", type=int, default=10_000,
                       help="Number of packets to send. Default: 10000")
    group.add_argument("--duration", "-d", type=float,
                       help="Seconds to send for (overrides --count)")
    ap.add_argument("--pps", type=float,
                    help="Packets per second (rate limit). If unset, send as fast as possible.")
    ap.add_argument("--no-connect", action="store_true",
                    help="Use sendto() instead of connect()+send()")
    args = ap.parse_args()

    send_packets(args.host, args.port,
                 payload_size=args.payload_size,
                 count=None if args.duration is not None else args.count,
                 pps=args.pps,
                 duration=args.duration,
                 connect_mode=not args.no_connect)
'''

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

    while (time.time() - start_time) < duration:
        packet_number = packet_number + 1

        packet[0] = (packet_number >> 24) & 0xFF
        packet[1] = (packet_number >> 16) & 0xFF
        packet[2] = (packet_number >> 8) & 0xFF
        packet[3] = (packet_number ) & 0xFF

        sock.sendto(packet, addr)

        bytes_sent_in_interval = bytes_sent_in_interval + packet_length

        if seconds_passed < int(time.time()):
            print(f"{interval_counter}-{interval_counter + 1}: {bytes_sent_in_interval / 1000} KB/s")
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
    print(f"Average Bytes  : {(send_bytes/interval)/1000} KB/s")
    #print(f"Packet #{packet_count} from {address}: {data.decode(errors='ignore')}")
    print(f"------------------------------------------------------------------------")

        


def client(sock, addr, packet_length, interval):


    #print(f"nettest: client listening on {local_address}! Send 'exit' to leave.")
    print("Do Ctrl+c to exit the program !!")
    print("UDP: sending Init to 127.0.0.1:1789")

    init_msg = b"Init\n"
    # send init msg to server 
    sock.sendto(init_msg, addr)

    addr_client = ("127.0.0.1", 12345)


    while True:
        data, addr = sock.recvfrom(BUFFER_SIZE)
        if not data:
            continue

        print(data.strip())

        if data.strip() == b"Init":
            return send_traffic(sock, addr, packet_length, interval)



def main():

    # define packet length
    packet_length = 64

    interval = 20.0

    # create the address for the server
    ip = "127.0.0.1"
    port = 12345
    local_address = (ip, port)

    server_address = (ip, 1798)


    # create the socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    # Bind the socket to the port
    sock.bind(local_address)
    # start the init exchange
    return client(sock, server_address, packet_length, interval)


if __name__ == "__main__":
    main()
