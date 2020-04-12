# this must be run inside a cloud-hypervisor VM
# started with --vsock cid=10,sock=/tmp/cid10.sock
import socket
import threading

PROXY_WELL_KNOWN_ADDR = 2      ## the host in VSOCK addressing
PROXY_WELL_KNOWN_PORT = 12345

DUMMY_RESPONSE = '''\
HTTP/1.1 200 OK
date: Sat, 11 Apr 2020 23:43:49 GMT
content-type: text/plain

ok
'''

def register_with_proxy(server_port):
    sock = socket.socket(socket.AF_VSOCK, socket.SOCK_STREAM, 0)
    sock.connect((PROXY_WELL_KNOWN_ADDR, PROXY_WELL_KNOWN_PORT))
    register_cmd = f"REGISTER {server_port}\n"
    sock.sendall(register_cmd.encode())
    print(f"registered {server_port}")
    threading.Thread(target=handle_keepalive, args=(sock,)).start()

def handle_keepalive(sock):
    while True:
        data = sock.recv(4)
        if data == b'ping'
            sock.sendall(b'pong')
        if data == b'':
            break
    sock.close()
    # and die?

def dummy_server():
    listen_sock = socket.socket(socket.AF_VSOCK, socket.SOCK_STREAM, 0)
    ## listen_sock.bind((-1, -1))
    # FIXME: the port should be dynamically bound like above, but fix it until subscription works
    listen_sock.bind((-1, 54321))
    listen_sock.listen()
    cid, port = listen_sock.getsockname()
    print(f"Listening on AF_VSOCK {cid}:{port}")
    register_with_proxy(port)
    while True:
        sock, addr = listen_sock.accept()
        threading.Thread(target=dummy_http_response, args=(sock, addr)).start()

def dummy_http_response(sock, addr):
    sock.recv(1000)  # ignore request
    sock.sendall(DUMMY_RESPONSE.encode())
    sock.close()


if __name__ == '__main__':
    dummy_server()
