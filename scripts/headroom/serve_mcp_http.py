#!/usr/bin/env python3
"""
headroom MCP stdio → HTTP adapter (PIP-584 Phase 1).

Wraps the headroom MCP server in stdio mode behind an HTTP server,
so that tools like mcpcall can reach it over HTTP.

Usage:
    python serve_mcp_http.py [--host 127.0.0.1] [--port 8765]
                             [--headroom-version 0.22.3]
"""

import argparse
import http.server
import json
import logging
import subprocess
import sys
import threading
import urllib.parse

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
)
logger = logging.getLogger("headroom-mcp-http")


class HeadroomMCPBridge:
    """Manages a headroom MCP subprocess and relays JSON-RPC messages."""

    def __init__(self, headroom_version: str):
        self.headroom_version = headroom_version
        self._process: subprocess.Popen | None = None
        self._lock = threading.Lock()

    def _ensure_process(self):
        if self._process is not None and self._process.poll() is None:
            return
        # Start headroom MCP server in stdio mode
        cmd = [
            sys.executable,
            "-m",
            "uv",
            "tool",
            "run",
            "--from",
            f"headroom-ai[proxy]=={self.headroom_version}",
            "headroom",
            "mcp",
        ]
        logger.info("Starting headroom MCP: %s", " ".join(cmd))
        self._process = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    def send_request(self, request: dict) -> dict:
        """Send a JSON-RPC request to the headroom MCP process and return the response."""
        self._ensure_process()
        payload = json.dumps(request).encode("utf-8")
        assert self._process is not None
        assert self._process.stdin is not None
        assert self._process.stdout is not None

        with self._lock:
            self._process.stdin.write(payload + b"\n")
            self._process.stdin.flush()
            line = self._process.stdout.readline()
            if not line:
                logger.error("MCP process closed stdout unexpectedly")
                return {"jsonrpc": "2.0", "error": {"code": -32000, "message": "Process closed"}, "id": request.get("id")}
            try:
                return json.loads(line)
            except json.JSONDecodeError as exc:
                logger.error("Invalid JSON from MCP: %s — raw: %s", exc, line[:200])
                return {"jsonrpc": "2.0", "error": {"code": -32700, "message": f"Parse error: {exc}"}, "id": request.get("id")}

    def close(self):
        if self._process and self._process.poll() is None:
            self._process.terminate()
            self._process.wait(timeout=5)


bridge: HeadroomMCPBridge | None = None


class MCPHTTPHandler(http.server.BaseHTTPRequestHandler):
    """HTTP handler that proxies JSON-RPC requests to headroom MCP."""

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path == "/health":
            self._json_response(200, {"status": "ok"})
        elif parsed.path == "/stats":
            if bridge is None:
                self._json_response(503, {"error": "bridge not initialized"})
                return
            resp = bridge.send_request({
                "jsonrpc": "2.0",
                "method": "headroom_stats",
                "params": {},
                "id": 1,
            })
            self._json_response(200, resp)
        else:
            self._json_response(404, {"error": f"not found: {parsed.path}"})

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path not in ("/mcp", "/api/mcp"):
            self._json_response(404, {"error": f"not found: {parsed.path}"})
            return

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)
        try:
            request = json.loads(body)
        except json.JSONDecodeError as exc:
            self._json_response(400, {"error": f"invalid JSON: {exc}"})
            return

        if bridge is None:
            self._json_response(503, {"error": "bridge not initialized"})
            return

        response = bridge.send_request(request)
        self._json_response(200, response)

    def _json_response(self, status: int, data: dict):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode("utf-8"))

    def log_message(self, fmt, *args):
        logger.info("HTTP %s — %s", self.path, fmt % args)


def main():
    parser = argparse.ArgumentParser(description="headroom MCP stdio → HTTP adapter")
    parser.add_argument("--host", default="127.0.0.1", help="bind address")
    parser.add_argument("--port", type=int, default=8765, help="bind port")
    parser.add_argument("--headroom-version", default="0.22.3", help="headroom-ai version")
    args = parser.parse_args()

    global bridge
    bridge = HeadroomMCPBridge(args.headroom_version)

    server = http.server.HTTPServer((args.host, args.port), MCPHTTPHandler)
    logger.info("Listening on http://%s:%d/mcp", args.host, args.port)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
        bridge.close()
        logger.info("Shutdown complete")


if __name__ == "__main__":
    main()
