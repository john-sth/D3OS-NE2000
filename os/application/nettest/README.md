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

## examples

```bash
# to start server mode (udp) :
nettest -u -l 10.0.2.15 1798

# client mode (udp) with 40 sec. duration and packet size 512:
nettest -u 10.0.2.15 1798 10.0.2.2 2000 40 512

```
