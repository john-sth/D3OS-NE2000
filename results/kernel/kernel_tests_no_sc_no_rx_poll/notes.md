this time no poll_sockets call in udp_send_traffic in benchmark.rs, and no scheduler.sleep in network/mod.rs
- expected behavior : a lot of Buffer Full error messages
