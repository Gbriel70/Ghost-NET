"""Real-process checks using only Python's standard library. Build before running."""
import pathlib
import socket
import subprocess
import threading
import time
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
BIN = ROOT / "target/debug/p2p-spike"
EVIDENCE = ROOT / "validation"


class Node:
    def __init__(self, name, *args):
        self.lines = []
        self.changed = threading.Condition()
        self.process = subprocess.Popen(
            [str(BIN), *args], stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            text=True, encoding="utf-8",
        )
        self.thread = threading.Thread(target=self.read, args=(name,), daemon=True)
        self.thread.start()

    def read(self, name):
        with (EVIDENCE / f"{name}.log").open("w") as log:
            for line in self.process.stdout:
                log.write(line)
                log.flush()
                with self.changed:
                    self.lines.append(line)
                    self.changed.notify_all()

    def wait(self, fragment, timeout=12):
        deadline = time.monotonic() + timeout
        with self.changed:
            while True:
                for line in self.lines:
                    if fragment in line:
                        return line
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise AssertionError(f"missing {fragment!r}: {''.join(self.lines)}")
                self.changed.wait(remaining)

    def address(self):
        return self.wait('event="listening"').split("address=", 1)[1].strip()

    def stop(self):
        if self.process.poll() is None:
            self.process.terminate()
        self.process.wait(timeout=5)
        self.thread.join(timeout=5)
        self.process.stdout.close()


class Processes(unittest.TestCase):
    def setUp(self):
        EVIDENCE.mkdir(exist_ok=True)
        self.nodes = []

    def tearDown(self):
        for node in reversed(self.nodes):
            node.stop()

    def node(self, suffix, dial=None):
        args = ["/ip4/127.0.0.1/tcp/0"]
        if dial:
            args.append(dial)
        node = Node(f"{self._testMethodName}-{suffix}", *args)
        self.nodes.append(node)
        return node

    def test_authenticated_ping_and_disconnect(self):
        a = self.node("a")
        address = a.address()
        b = self.node("b", address)
        a.wait('event="connected"')
        connected = b.wait('event="connected"')
        self.assertIn(address.rsplit("/", 1)[1], connected)
        b_id = b.address().rsplit("/", 1)[1]
        self.assertNotEqual(address.rsplit("/", 1)[1], b_id)
        self.assertIn(b_id, a.wait('event="connected"'))
        a.wait('event="ping_ok"')
        b.wait('event="ping_ok"')
        a.process.kill()
        a.process.wait(timeout=5)
        b.wait('event="disconnected"')
        self.assertIsNone(b.process.poll())

    def test_wrong_expected_identity(self):
        a = self.node("a")
        address = a.address()
        other = self.node("other")
        wrong_id = other.address().rsplit("/", 1)[1]
        b = self.node("b", address.rsplit("/", 1)[0] + "/" + wrong_id)
        error = b.wait('event="dial_error"')
        self.assertIn("WrongPeerId", error)
        self.assertFalse(any('event="connected"' in line for line in b.lines))
        self.assertIsNone(b.process.poll())

    def test_peer_absent(self):
        with socket.socket() as reserved:
            reserved.bind(("127.0.0.1", 0))
            port = reserved.getsockname()[1]
            # Bound but not listening: no other process can claim this port.
            b = self.node("b", f"/ip4/127.0.0.1/tcp/{port}")
            b.wait('event="dial_error"')
            self.assertIsNone(b.process.poll())

    def test_connection_limit(self):
        a = self.node("a")
        address = a.address()
        for index in range(4):
            peer = self.node(str(index), address)
            peer.wait('event="ping_ok"')
        extra = self.node("extra", address)
        error = a.wait('event="incoming_error"')
        self.assertIn("Exceeded", error)
        self.assertIsNone(a.process.poll())
        self.assertIsNone(extra.process.poll())

    def test_handshake_timeout(self):
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            listener.listen(1)
            listener.settimeout(10)
            b = self.node("b", f"/ip4/127.0.0.1/tcp/{listener.getsockname()[1]}")
            started = time.monotonic()
            connection, _ = listener.accept()
            with connection:
                error = b.wait('event="dial_error"', timeout=10)
                self.assertRegex(error.lower(), r"timeout|timedout|timed out")
                self.assertLess(time.monotonic() - started, 10)
                self.assertIsNone(b.process.poll())

    def test_remote_failure_does_not_prevent_new_connection(self):
        a = self.node("a")
        address = a.address()
        other = self.node("other")
        wrong_id = other.address().rsplit("/", 1)[1]
        b = self.node("b", address.rsplit("/", 1)[0] + "/" + wrong_id)
        b.wait('event="dial_error"')
        c = self.node("c", b.address())
        c.wait('event="ping_ok"')
        self.assertIsNone(b.process.poll())


if __name__ == "__main__":
    unittest.main(verbosity=2)
