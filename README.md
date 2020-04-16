# `subscription-proxy-vsock`

The goal is to have a HTTP proxy that listens to http requests,
and forwards them to applications running in a VM over virtio-vsock.

Ideally, I would start any number of micro VM apps and they would automatically register with the proxy
(atypical to most proxies/load-balancers, but similar to mongrel2 or the uwsgi fastrouter/subscription).

http -> [vsock-proxy-lb] <- (== vm vsock ==) <- [vsock client connector + gunicorn]

The virtual machine will run with cloud-hypervisor started with `--vsock cid=10,sock=/tmp/cid10.sock`… TBD!


## Roadmap

A rough roadmap is 
https://github.com/gdamjan/subscription-proxy-vsock/milestones?with_issues=no

0.1)
  listeners for http and vsock (or its emulation)

0.2)
  ping/pong protocol

0.3)
  http request, http response in protocol

0.4)
  register cmd (regex: hostname/path)

2.0)
  tls

3.0)
  websockets
  request websocket:<uuid> ->
  <- connect websocket:<uuid>

4.0)
  http2

5.0)
  vm manager (cgi like)


## Links:

http server:
- https://docs.rs/httparse/
- https://docs.rs/async-h1/

Request/Response serialization candidates:
- https://github.com/sbdchd/tnetstring/
- https://github.com/capnproto/capnproto-rust

cloud-hypervisor:
- https://github.com/cloud-hypervisor/cloud-hypervisor
- https://github.com/firecracker-microvm/firecracker/blob/master/docs/vsock.md

VSock:
- https://wiki.qemu.org/Features/VirtioVsock
- http://kvmonz.blogspot.com/p/knowledge-using-vsock.html
- http://man7.org/linux/man-pages/man7/vsock.7.html
