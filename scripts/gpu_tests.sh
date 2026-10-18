#!/usr/bin/env bash
# Run every GPU-bound referee: the ignored `render` tests and the golden
# check. SERIALISED — each test creates its own wgpu device and the harness's
# parallel threads make that SIGSEGV on the Intel iGPU driver (2 of 3 runs
# measured 2026-09-01). `OPENBF_ADAPTER=igpu|dgpu` picks the GPU.
#
#   scripts/gpu_tests.sh            # render GPU tests + goldens --check
#   scripts/gpu_tests.sh --render   # only the render crate's GPU tests
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/engine"

cargo test -p render -- --ignored --test-threads=1
[ "${1:-}" = "--render" ] && exit 0
exec "$ROOT/scripts/goldens.sh" --check
