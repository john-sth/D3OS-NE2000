```rust
+-+-+-+-+-+-+
|N|e|2|0|0|0|
+-+-+-+-+-+-+
```

# NE2000 Network Card Driver

## Files which have been added and modified for this thesis:

- **ROOT DIR**:

  - Cargo.toml: added nettest
  - added nettest.py and netcat.sh, send_packet.sh for testing
  - configured qemu-pci.sh for rtl8029as
  - dir results contains log output, .dump files and screenshots from the benchmark tests
  - Makefile.toml : added emulated ne2k_pci
  - qemu-pci.sh : added device and vendor id
    - how to find kernel module : modinfo ne2k-pci
    - bus id: lspci -nnk | grep -A3 -i ne2k-pci
  - Cargo.toml Card Driver for D3OS

- **KERNEL**:

  - os/kernel/src/device/ne2k added
  - os/kernel/src/mod.rs : modified
  - os/kernel/src/device : added module ne2k, modified mod.rs
  - os/kernel/srcnetwork: modified mod.rs, added support for ne2000
  - os/kernel/src/boot.rs : comments in the section network, old code
    - contains codes for threads for starting network benchmark tests
  - consts.rs : increased the kernel heap page size

- **APPLICATION**:
  - os/application/nettest : benchmark tool

## MISC

### Driver Implementation

#### Queues:

- send_queue

- MPSC queue
- wrapped into a Mutex
- producer = Sender,

- consumer = receiver
  - Mutex ensures exclusive access when an interrupt handler and a polling path might drain the queue
- every TxToken can enqueue a DMA buffer without blocking,
  while the driver’s service loop dequeues those buffers and
  recycles the memory when the card finishes transmission.

#### receive_buffer_empty

#### BNDY and CURR Register

- BNDY : read pointer, first page not yet processed, driver owned
- all pages up to but not including the packet which is being filled get freed
- CURR : first page page being written , hardware owned
- BNDY also used when packet is removed
- CURR = "where the NIC will write next" ← hardware write pointer
- BNRY = "last page I've already handled" ← driver read pointer

- When both point to the same page the ring is empty;
  when CURR catches up to BNRY the ring is full and reception stops to avoid overwriting unread data

Observation

- rtl8139 has no problems with sending a lot of packets (2000)
- if i use the ne2k, i get the buffer full error

#### changes

edited const.rs -> increase
//===============================================================================================
// update 15.08.2025: increase heap page size (as suggested by M. Schoettner)
//const INIT_HEAP_PAGES: usize = 0x400; // number of heap pages for booting the OS (old value)
//===============================================================================================
const INIT_HEAP_PAGES: usize = 0x4000; // number of heap pages for booting the OS

### Errors

#### Reason for slirp errors:

- because SLIRP is slow and smoltcp is single‑threaded. A huge UDP RX queue makes each poll() spend lots of time shoveling receive packets (holding the sockets lock), so egress doesn’t get serviced fast enough.
  When rx_size was shranked from 1000 → 2, the work poll() does on ingress per tick was limited, freeing time for TX to drain, so your burst to the host stopped tripping SLIRP’s “failed to send packet” path.

- QEMU “user” networking (SLIRP) is a userspace NAT with poor throughput and small queues. It’s convenient but explicitly documented as “a lot of overhead so the performance is poor.” Bursts from the guest are easy to drop/log as errors on the host side.
  wiki.qemu.org

- smoltcp drives both RX and TX in the same Interface::poll() loop. Big RX buffers mean poll() can enqueue many datagrams into UDP socket before it ever gets back to egress. That increases lock hold time on the global SocketSet and pushes out TX work. (See iface docs: poll() is the driver for interface logic.)

- Oversized buffers can reduce throughput. There’s even an open smoltcp issue showing a “sweet spot” where increasing buffer sizes past a point hurts performance due to extra work and cache pressure.

- Symptom on the host: SLIRP will complain (e.g., “Failed to send packet, ret: -1”) when it can’t keep up.
- Reducing RX queue shortened each poll cycle, letting TX keep pace and avoiding SLIRP’s error path.

- improvement : dropping rx_size to 2 :

- guest now drops excess inbound datagrams earlier (socket RX buffer fills quickly),
- which makes each poll() iteration shorter,
- which gives more CPU to egress,
- which reduces the burst pressure on SLIRP and avoids its send‑fail log spam.
- Keep RX modest, find a middle ground that matches poll rate.
- Increase TX payload slab (total bytes) rather than cranking metadata counts, and poll more frequently (or use poll_delay() for tight pacing). That helps TX drain smoothly without starving the system.
- For serious throughput tests, switch QEMU from SLIRP to tap/bridge networking; it bypasses SLIRP’s userspace NAT bottleneck. QEMU’s docs call out the backend options.
- TL;DR: smaller RX limited per‑tick ingress work, unblocked TX, and side‑stepped SLIRP’s bottleneck—so your bursts look “faster” and cleaner.

https://github.com/smoltcp-rs/smoltcp/issues/949?utm_source=chatgpt.com

## TODO:

### for the driver

- [x] nettest send benchmark create table
- [x] nettest receive benchmark create table
- [x] readme file how to integrate an emulated nic to d3os, add qemu network site
- [x] add a list of modified and used files in the OS
- [x] rewrite of nettest application in rust
- [x] create flowchart for qemu network
- [x] rewrite the call for receive and overflow with AtomicBool values
- [x] check overwrite method
- [x] check boundaries for receive buffer
- [x] check page size page buffer
- [x] check if the ovwe bit gets actually set at the initialization of the nic
- [x] check if packets bigger than max. Ethernet size don't get processed in the receive method and if so do the same for smoltcp so that no buffer gets enqueued
- [x] check network/mod.rs open_socket() for transmit and rx size for packetbuffer
- [x] clean up code
- [x] reread code, check for commenting what the return value of a function is
- [x] create flowchart queues
- [x] latex add receive error img in pdf
- [x] nettest for rtl8139?
- [x] execute benchmark.rs
- [ ] write nettest documentation in README.md
- [ ] fix latex code section error
- [ ] READ https://en.wikipedia.org/wiki/Ethernet_frame
- [ ] reread fifo breq underrun, overrun
- [ ] maybe add tcp to nettest (at the end)
- [ ] create the presentation
- [ ] remove unneeded logs and todos and unused warnings!!
- [ ] bring main up to date with development
- [ ] upload code and thesis until friday !!!!!

### for the presentation

- use class diagram
- one page for d3os, purpose, name rtl8139 card, network features
- ne2000 architecture, get the information from the thesis:
  - general information about the card + image
  - page registers
  - fifo
  - ring buffer
- the driver:
  - init procedure (just explain that certain registers get set)
  - explain in short smoltcp Token logic
  - and Interrupts of the NIC
  - send and receive part
  - overflow function
- evaluation and benchmarks
  - show maybe a table of transmission rate
  - if possible create video/gif recording
    from pyplot for the transmission rate
  - talk about results and further things which could be implemented
  - prepare for questions after the presentation
