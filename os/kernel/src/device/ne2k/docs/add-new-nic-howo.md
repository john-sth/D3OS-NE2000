# How to add a emulated Network Interface Card to D3OS in QEMU

## 1. select the preferred nic model from the QEMU network section https://en.wikibooks.org/wiki/QEMU/Devices/Network

## 2. Go to the main directory of D3OS and open **Makefile.toml**

## 3. Go to the section **[tasks.qemu]** and add the required fields for the nic emulation

## 4. Example Code for a emulated NE2000 NIC :

```toml
    "-nic",
    "model=ne2k_pci,id=ne2k,hostfwd=udp::1798-:1798",
    "-object",
    "filter-dump,id=filter0,netdev=ne2k,file=ne2k.dump",
```

- "-nic" : create a NIC together with a host backend by using the -nic parameter.
- model : selected model as listed in the QEMU Network Devices Section
- id : id by which the emulated device is identified, free to choose
- hostfwd: host forwarding, specify the port, which to bind the nic to
  - hostfwd=GUEST_PORT:HOST_PORT
  - you can also add instead a range, for example 1798-1820:1798
  - to send a network packet outside of the guest environment send the packet to the ip address 10.0.2.2, QEMU's default gateway.
  - More on this topic here: https://wiki.qemu.org/Documentation/Networking
  - QEMU will then automatically redirect the packets to the specified HOST_PORT on the local network
- optionally add the socket type
- object: add Network Monitoring
  - filter-dump : capture network traffic
  - id : free to choose
  - netdev : add the id from the "-nic" section from earlier, in this example ne2k
  - specify the file name where to dump the network traffic
- after capturing packets the resulting dump file can be opened with Wireshark for network analysis

## 5. optionally add the Code snippet mentioned above to **[tasks.qemu]** in the **Makefile.toml** to add the Network Device when running D3OS in debug mode
