#!/usr/bin/env python3
import argparse
import os
import socket
import time

BUFFER_SIZE = 65535

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
        # Optional but faster: avoids passing addr on every send
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
            header = seq.to_bytes(4, byteorder="big")
            payload = header + (os.urandom(payload_size - 4) if payload_size > 4 else b"")
            if connect_mode:
                sock.send(payload)
                #time.sleep(0.2)
            else:
                sock.sendto(payload, addr)
                #time.sleep(0.2)

            sent += 1
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
        stop_msg = b"exit"
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

if __name__ == "__main__":
    main()
