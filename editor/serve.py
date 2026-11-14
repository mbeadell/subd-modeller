"""Serve the local editor and evaluate previews with the Rust CLI."""
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
BINARY = ROOT.parent / "target/debug/subd-prototype"


class Editor(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != "/":
            self.send_error(404)
            return
        self.reply("text/html; charset=utf-8", (ROOT / "index.html").read_bytes())

    def do_POST(self):
        if self.path not in ("/preview", "/spline-step", "/faceted-step", "/stl"):
            self.send_error(404)
            return
        try:
            if self.headers.get_content_type() != "application/json":
                raise ValueError("Expected JSON")
            size = int(self.headers.get("Content-Length", "0"))
            if not 0 < size <= 32768:
                raise ValueError("Invalid request size")
            payload = json.loads(self.rfile.read(size))
            width = 2
            quads = None
            creases = []
            if self.path == "/preview" and isinstance(payload, list):
                points = payload
            else:
                if not isinstance(payload, dict):
                    raise ValueError("Expected cage and width")
                points = payload.get("points")
                quads = payload.get("quads")
                creases = payload.get("creases", [])
                width = 2 if self.path == "/preview" else payload.get("width")
                if type(width) not in (int, float) or not 1e-9 <= width <= 1e12:
                    raise ValueError("Invalid width")
            level = payload.get("level") if self.path == "/stl" else 0
            if type(level) is not int or not 0 <= level <= 5:
                raise ValueError("Invalid mesh level")
            if not isinstance(points, list) or len(points) not in (range(8, 33, 4) if quads is None else range(8, 129)):
                raise ValueError("Expected two to eight sections")
            for point in points:
                if not isinstance(point, list) or len(point) != 3 or any(
                    type(value) not in (int, float) or not abs(value) <= 1e12
                    for value in point
                ):
                    raise ValueError("Invalid control point")
            if quads is not None and (
                not isinstance(quads, list) or not 6 <= len(quads) <= 126 or any(
                    not isinstance(face, list) or len(face) != 4 or any(
                        type(index) is not int or not 0 <= index < len(points)
                        for index in face
                    ) for face in quads
                )
            ):
                raise ValueError("Invalid quads")
            if not isinstance(creases, list) or len(creases) > 252 or any(
                not isinstance(edge, list) or len(edge) != 2 or any(
                    type(index) is not int or not 0 <= index < len(points)
                    for index in edge
                ) for edge in creases
            ):
                raise ValueError("Invalid creases")
            with tempfile.TemporaryDirectory() as directory:
                cage = Path(directory) / "input.cage"
                result = Path(directory) / "result"
                header = "subd-sections 1" if quads is None else "subd-quads 1"
                cage.write_text(f"{header}\nwidth-mm {width}\n" + "".join(
                    " ".join(map(str, point)) + "\n" for point in points
                ) + "".join("q " + " ".join(map(str, face)) + "\n" for face in (quads or []))
                    + "".join("c " + " ".join(map(str, edge)) + "\n" for edge in creases))
                commands = {
                    "/stl": ["stl", cage, str(level), result],
                    "/preview": ["preview", cage, "3", result],
                    "/spline-step": ["spline-step", cage, result],
                    "/faceted-step": ["step", cage, "2", result],
                }
                subprocess.run(
                    [BINARY, *commands[self.path]],
                    check=True, capture_output=True, timeout=5,
                )
                self.reply("application/octet-stream" if self.path == "/stl" else "text/plain", result.read_bytes())
        except (ValueError, UnicodeError):
            self.send_error(400, "Invalid cage")
        except subprocess.CalledProcessError as error:
            self.reply("text/plain; charset=utf-8", error.stderr.strip() or b"Export failed", status=422)
        except (OSError, subprocess.SubprocessError):
            self.send_error(500, "Rust preview failed")

    def reply(self, content_type, body, status=200):
        self.send_response(status)
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args):
        pass


if __name__ == "__main__":
    if not BINARY.is_file():
        raise SystemExit("Build the Rust prototype with cargo build first.")
    with ThreadingHTTPServer(("127.0.0.1", 8765), Editor) as server:
        print("Open http://127.0.0.1:8765 in your browser", flush=True)
        server.serve_forever()
