```rust
                __  __            __
   ____  ___  / /_/ /____  _____/ /_
  / __ \/ _ \/ __/ __/ _ \/ ___/ __/
 / / / /  __/ /_/ /_/  __(__  ) /_
/_/ /_/\___/\__/\__/\___/____/\__/
```

# nettest: Client-/Server-Application for testing TCP/UDP Connections

## Usage

```bash
nettest [-u] [-l] HOST PORT [REMOTE_HOST] [REMOTE_PORT] [duration] [packet_length]
```

- **u**: Specify the protocol (UDP/TCP) \n
- **l**: **Server** mode, listen on HOST:PORT. \n

- **duration**: for **client** mode, specify time for how long to send packets.

- **packet_length** : for **client** mode, specify the preferred length of the packets, which should be transmitted."

### examples

```bash
# to start server mode (udp) :
nettest -u -l 10.0.2.15 1798

# client mode (udp) with 40 sec. duration and packet size 512:
nettest -u 10.0.2.15 1798 10.0.2.2 2000 40 512

```

## python implementation for the host OS

```bash
python nettest.py <local_address> <local_port> <remote_address> <remote_port> [--mode MODE] [--packet_length BYTES] [--timing_interval MS]
```

### Arguments

```bash
<local_address> – The IP address to bind locally (e.g., 127.0.0.1).

<local_port> – The local UDP port to listen on (e.g., 2000).

<remote_address> – The remote IP address to send to (e.g., 127.0.0.1).

<remote_port> – The remote UDP port to send to (e.g., 1798).
```

### Optional Flags

```bash
--mode

0 (default): Server mode (listens for incoming packets).

1: Client mode (sends packets to the remote server).

--packet_length

Length of UDP packets in bytes (default depends on script; e.g., 64).

--timing_interval

Interval (in ms) between sending packets (client mode only).
```

### examples

run as server:

```bash
python nettest.py 127.0.0.1 2000 127.0.0.1 1798
```

run as client:

```bash
python nettest.py 127.0.0.1 2000 127.0.0.1 1798 --mode 1 --packet_length 64

```
