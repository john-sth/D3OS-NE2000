# without a delay the os crashed, a lot of failed to send packet errors
# incremenally decreased the dealy value up to 0.005 seconds in nettest.py then slirp errors again and error dequeuing: Empty
# run test again with 0.007 delay : worked
# crashed again at 256 packet length with error Error dequeuing: Empty, increased the RECEIVE_QUEUE_CAP to 512 -> worked 
# crashed again at 1024: increase cap again
-> worked again
# crashed again at 1450: increase cap again to 1500
-> worked again
after reviewing this results, the observation is that not qemu but the queue is the problem -> run initial tests again without delay
cap = 1500 
no delay in nettest.py
-> didnt work slirp errors still persist
