"""Exercise an EMPTY running party server with 8 real TCP clients (stdlib only).

Usage: python scripts/test-party-server.py [IP] [PORT]
The test starts a match, verifies authoritative state, then disconnects everyone.
"""
import json
import socket
import struct
import sys
import time


def send(sock, value):
    data = json.dumps(value).encode()
    sock.sendall(struct.pack("!I", len(data)) + data)


def read_exact(sock, length):
    data = b""
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk:
            raise RuntimeError("Unexpected disconnect")
        data += chunk
    return data


def until(sock, kind):
    for _ in range(200):
        length = struct.unpack("!I", read_exact(sock, 4))[0]
        assert 0 < length <= 2 * 1024 * 1024
        message = json.loads(read_exact(sock, length))
        if kind in message:
            return message[kind]
        if "Error" in message:
            raise AssertionError(message["Error"])
    raise AssertionError("Expected " + kind)


def connect(address, name, protocol=1):
    sock = socket.create_connection(address, timeout=5)
    sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
    send(sock, {"Join": {"protocol": protocol, "name": name}})
    return sock


def main():
    address = (sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1", int(sys.argv[2]) if len(sys.argv) > 2 else 7777)
    clients = []
    try:
        with connect(address, "WrongVersion", 999) as wrong:
            assert "versions" in until(wrong, "Error")
        for i in range(8):
            client = connect(address, "Test" + str(i))
            clients.append(client)
            until(client, "Welcome")
            lobby = until(client, "Lobby")
            assert len(lobby["members"]) == i + 1, "Use an empty test server"
            send(client, {"Weapon": i % 5})
            send(client, {"Ready": True})
        with connect(address, "Ninth") as ninth:
            assert "full" in until(ninth, "Error")
        while True:
            lobby = until(clients[0], "Lobby")
            if len(lobby["members"]) == 8 and all(m["ready"] for m in lobby["members"]):
                break
        send(clients[1], "Start")
        assert "host" in until(clients[1], "Error")
        send(clients[0], "Start")
        for i, client in enumerate(clients):
            started = until(client, "Started")
            assert len(started["members"]) == 8
            state = until(client, "Snapshot")
            assert len(state["players"]) == 8
            assert state["players"][i]["weapon"] == i % 5
            assert all(p["health"] == 100.0 for p in state["players"])
        with connect(address, "Late") as late:
            assert "running" in until(late, "Error")
        # Input is not allowed to change the starting weapon.
        send(clients[0], {"Input": {"movement": [0.0, 0.0], "aim": [200.0, 200.0], "fire": False, "dash": False, "reload": False, "weapon": 4}})
        for _ in range(3):
            state = until(clients[0], "Snapshot")
        assert state["players"][0]["weapon"] == 0
        clients[0].close()
        lobby = until(clients[1], "Lobby")
        assert lobby["host"] == lobby["members"][0]["id"]
        assert len(lobby["members"]) == 7
        print("PASS: Linux/server handshake, eight clients, full room, host-only start, weapon lock, late join rejection, host transfer")
    finally:
        for client in clients:
            client.close()
    time.sleep(0.2)
    with connect(address, "FreshRoom") as client:
        until(client, "Welcome")
        lobby = until(client, "Lobby")
        assert not lobby["in_game"] and len(lobby["members"]) == 1
    print("PASS: room resets after everyone leaves")


if __name__ == "__main__":
    main()
