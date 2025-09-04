removed the scheduler.sleep line in network/mod.rs for the poll thread
the poll_sockets was also still in the benchmark.rs send_traffic function
- there were a lot of Buffer full messages in the log while sending, see image
