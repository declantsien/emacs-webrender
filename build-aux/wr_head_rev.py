#!/usr/bin/env python3

import os
import subprocess
import sys
import pytoml as toml

# MESON_SOURCE_ROOT = os.environ.get('MESON_SOURCE_ROOT')
# raise error if MESON_SOURCE_ROOT not found
MESON_SOURCE_ROOT = os.environ['MESON_SOURCE_ROOT']
CARGO_LOCK = os.path.join(MESON_SOURCE_ROOT, "Cargo.lock")
rev_sha = ''

if os.path.exists(CARGO_LOCK):
    with open(CARGO_LOCK) as lock_file:
        obj = toml.load(lock_file)
        filtered = filter(lambda pkg: pkg["name"] == "webrender", obj["package"])
        wr_pkg = list(filtered)[0]
        rev_sha = wr_pkg["source"].split("#")[1]
else:
    CARGO_TOML = os.path.join(MESON_SOURCE_ROOT, "ports/webrender/Cargo.toml")
    with open(CARGO_TOML) as f:
        obj = toml.load(f)
        rev_sha = obj["dependencies"]["webrender"]["rev"]

sys.stdout.write(rev_sha)
# sys.stdout.write(f"{rev_sha}\n")
