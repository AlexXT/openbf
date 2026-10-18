#!/usr/bin/env python3
"""scout_look.py — one shot: ask the local vision model about an existing PS2 vs port pair.

The model does not write code and does not edit FINDINGS.md. Each call is a fresh
request (no chat history). A director reads the JSON and decides what to keep.

  python3 scripts/scout_look.py bf1_kas2 cp5_cp10_dock_one_spawn
  python3 scripts/scout_look.py --model qwen3-vl:8b --no-save bf1_kas2 cp4_cp11_dock_two_spawn

Writes one JSON line to engine/artifacts/loop/scout/inbox.jsonl unless --no-save.
Refuses to start while the Opus autoloop or PCSX2 is running (they share the GPU).
"""
import argparse, base64, io, json, os, subprocess, sys, time, urllib.error, urllib.request
from PIL import Image, ImageDraw

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
REF = os.path.join(ROOT, "engine/artifacts/pcsx2/ref")
INBOX = os.path.join(ROOT, "engine/artifacts/loop/scout/inbox.jsonl")
OLLAMA = os.environ.get("OLLAMA_HOST", "http://127.0.0.1:11434")
DEFAULT_MODEL = "qwen3-vl:8b-instruct"

RULES = """The first image is labeled PS2. The second image is labeled PORT. Same camera pose.
Ignore sharpness, resolution, the HUD, and the player character. The port leaves those out on purpose.
Say PS2 or PORT, never "image 1" or "image 2". Use this shape:

water: <colour and transparency on PS2> vs <colour and transparency on PORT>
horizon: <PS2 horizon> vs <PORT horizon, including any coloured band>
terrain: <sand, grass, or stone, and which side has which>
missing: <objects on one side that are absent on the other, excluding HUD and the player, or "none">
"""


def busy():
    """The coding loop and PCSX2 use the same GPU. A scout call waits until they are gone."""
    try:
        out = subprocess.run(["ps", "-eo", "args"], capture_output=True, text=True, check=True).stdout
    except (OSError, subprocess.CalledProcessError):
        return None
    for line in out.splitlines():
        if "scripts/autoloop.sh" in line or "PCSX2" in line or "pcsx2" in line:
            return line.strip()[:160]
    return None


def labeled_b64(path, title):
    """Stamp a caption on the frame. The model swaps which side is which if the pictures are unlabeled."""
    im = Image.open(path).convert("RGB")
    canvas = Image.new("RGB", (im.width, im.height + 24), (0, 0, 0))
    canvas.paste(im, (0, 24))
    ImageDraw.Draw(canvas).text((8, 6), title, fill=(255, 220, 0))
    buf = io.BytesIO()
    canvas.save(buf, format="PNG")
    return base64.b64encode(buf.getvalue()).decode("ascii")


def ask(model, ps2_path, port_path):
    body = {
        "model": model,
        "stream": False,
        "keep_alive": "10m",
        "options": {"temperature": 0.2, "num_ctx": 8192, "num_predict": 500},
        "messages": [
            {
                "role": "user",
                "content": RULES,
                "images": [labeled_b64(ps2_path, "PS2"), labeled_b64(port_path, "PORT")],
            }
        ],
    }
    req = urllib.request.Request(
        OLLAMA + "/api/chat",
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
    )
    started = time.monotonic()
    try:
        with urllib.request.urlopen(req, timeout=600) as resp:
            payload = json.load(resp)
    except urllib.error.HTTPError as e:
        detail = e.read().decode("utf-8", "replace")
        raise SystemExit(f"ollama HTTP {e.code}: {detail}") from e
    elapsed = time.monotonic() - started
    message = payload.get("message") or {}
    raw = message.get("content") or ""
    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError:
        parsed = None
    return elapsed, payload, raw, parsed


def main():
    ap = argparse.ArgumentParser(description="Ask the local vision model about one PS2/port pair")
    ap.add_argument("map")
    ap.add_argument("spot")
    ap.add_argument("--model", default=DEFAULT_MODEL)
    ap.add_argument("--no-save", action="store_true")
    args = ap.parse_args()

    holder = busy()
    if holder:
        raise SystemExit(f"GPU is in use, not calling the vision model:\n{holder}")

    ps2 = os.path.join(REF, args.map, args.spot + ".png")
    port = os.path.join(REF, args.map, "port", args.spot + ".png")
    for path in (ps2, port):
        if not os.path.isfile(path):
            raise SystemExit(f"missing image: {path}")

    elapsed, payload, raw, parsed = ask(args.model, ps2, port)
    record = {
        "ts": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "model": args.model,
        "map": args.map,
        "spot": args.spot,
        "ps2": os.path.relpath(ps2, ROOT),
        "port": os.path.relpath(port, ROOT),
        "seconds": round(elapsed, 1),
        "eval_count": payload.get("eval_count"),
        "prompt_eval_count": payload.get("prompt_eval_count"),
        "status": "unreviewed",
        "result": parsed if parsed is not None else {"raw": raw},
    }
    text = json.dumps(record, indent=2)
    print(text)
    if not args.no_save:
        os.makedirs(os.path.dirname(INBOX), exist_ok=True)
        with open(INBOX, "a", encoding="utf-8") as f:
            f.write(json.dumps(record) + "\n")
        print(f"\nappended {INBOX}", file=sys.stderr)


if __name__ == "__main__":
    main()
