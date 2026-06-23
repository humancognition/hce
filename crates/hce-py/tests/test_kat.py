import json
import os
import sys

import hce

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
KAT_DIR = os.path.join(PROJECT_ROOT, "shared", "kat")
KEY = b"hce-kat-standard-key-32-bytes!!"

total = 0
failures = 0

for name in ("universal", "eu", "en", "numeric"):
    path = os.path.join(KAT_DIR, f"{name}.json")
    with open(path) as f:
        data = json.load(f)

    level = name
    for v in data["vectors"]:
        uuid_hex = v["uuid"]
        mode = v["mode"]
        expected = v["hce"]

        uuid_bytes = bytes.fromhex(uuid_hex)

        if mode == "plain":
            h = hce.Hce(key=None, level=level, mode="plain")
        elif mode == "open":
            h = hce.Hce(key=KEY, level=level, mode="open")
        else:
            h = hce.Hce(key=KEY, level=level, mode="sealed")

        encoded = h.encode(uuid_bytes)
        total += 1
        if encoded != expected:
            print(f"  {name}[{v['index']}] encode mismatch", file=sys.stderr)
            failures += 1
            continue

        decoded = h.decode(encoded)
        total += 1
        if decoded != uuid_bytes:
            print(f"  {name}[{v['index']}] decode mismatch", file=sys.stderr)
            failures += 1
            continue

    print(f"  {name}: OK ({len(data['vectors'])} vectors)")

if failures:
    print(f"FAIL: {failures} of {total} checks failed", file=sys.stderr)
    sys.exit(1)
print(f"ALL {total} checks passed")
