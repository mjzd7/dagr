#!/usr/bin/env python3
"""Minimal scripted LSP server for bridge tests: answers every
textDocument/references request with two canned locations."""
import json, sys, os

def recv():
    headers = {}
    while True:
        line = sys.stdin.readline()
        if not line or line in ("\r\n", "\n"):
            break
        if ":" in line:
            k, v = line.split(":", 1)
            headers[k.strip().lower()] = v.strip()
    n = int(headers.get("content-length", 0))
    return json.loads(sys.stdin.read(n)) if n else None

def send(obj):
    body = json.dumps(obj).encode()
    sys.stdout.buffer.write(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)
    sys.stdout.flush()

while True:
    msg = recv()
    if msg is None:
        break
    method = msg.get("method")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": msg["id"], "result": {"capabilities": {}}})
    elif method == "initialized":
        pass
    elif method == "shutdown":
        send({"jsonrpc": "2.0", "id": msg["id"], "result": None})
        break
    elif method == "textDocument/references":
        send({"jsonrpc": "2.0", "id": msg["id"], "result": [
            {"uri": "file:///fake/caller.rs",
             "range": {"start": {"line": 9, "character": 0},
                       "end": {"line": 9, "character": 5}}},
            {"uri": "file:///fake/other.rs",
             "range": {"start": {"line": 41, "character": 3},
                       "end": {"line": 41, "character": 8}}},
        ]})
