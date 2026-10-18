#!/usr/bin/env python3
"""loop_cpu.py - keep the autoloop off the maintainer's fast cores while he is at the machine (2026-10-11).

The loop and everything it starts live in the systemd user slice `openbf.slice` (scripts/start_loop.sh, and the
scripts/loopbin/systemd-run shim for the scopes sessions open). That slice has a low CPUWeight, which decides who
wins a CONTENDED core - but measured, 32 busy loop threads slow 8 of the maintainer's jobs 1.7x at any weight (shared
physical cores, lost turbo), and confined to the efficiency cores they cost him nothing (0.52 s against 0.58 s alone).
So this governor sets the CPU affinity of every thread in the slice:

  POLITE  the maintainer is here (lid open, screen unlocked, input in the last OPENBF_IDLE_SECS): the efficiency cores only
          (/sys/devices/cpu_atom/cpus, here 16-31), leaving the 8 performance cores to him.
  FULL    the lid is closed, the screen is locked, or no keyboard / touchpad / mouse interrupt for OPENBF_IDLE_SECS
          (default 300 s): every CPU.

PCSX2 itself is always given every CPU: its emulation must hold full speed (captures press keys on the wall clock).

  loop_cpu.py watch     run until no autoloop.sh is left (start_loop.sh starts it inside the loop's scope)
  loop_cpu.py once      apply the current mode once and print it (OPENBF_FORCE_MODE=polite|full to force one)
  loop_cpu.py status    print the mode file

Knobs (environment): OPENBF_POLITE=0 turns the affinity off (the slice's weight stays); OPENBF_POLITE_CPUS="16-31";
OPENBF_IDLE_SECS=300. Idle is read from /proc/interrupts (i8042 keyboard / trackpoint, the touchpad's line, and
xhci above its background rate for a USB or Bluetooth mouse): KDE on Wayland gives no idle time and does not set
logind's IdleHint. A source that cannot be read counts as "the maintainer is here".
"""
import glob, os, re, subprocess, sys, time

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
LOOP = os.path.join(ROOT, "engine/artifacts/loop")
MODE_FILE = os.path.join(LOOP, "cpu_mode")
SLICE = "openbf.slice"
POLL = 5.0
XHCI_RATE = 60.0     # interrupts / s: this machine's xhci idles at about 13 / s; a moving mouse is 125 / s and up


def parse_cpus(s):
    out = set()
    for part in s.replace("\n", "").split(","):
        if not part.strip():
            continue
        a, _, b = part.partition("-")
        out |= set(range(int(a), int(b or a) + 1))
    return out


def all_cpus():
    try:
        return parse_cpus(open("/sys/devices/system/cpu/online").read())
    except OSError:
        return set(range(os.cpu_count() or 1))


def polite_cpus():
    full = all_cpus()
    env = os.environ.get("OPENBF_POLITE_CPUS")
    if env:
        c = parse_cpus(env) & full
    else:
        try:
            c = parse_cpus(open("/sys/devices/cpu_atom/cpus").read()) & full   # the efficiency cores of a hybrid CPU
        except OSError:
            c = set(sorted(full)[len(full) // 2:])                              # no hybrid CPU: the upper half
    return c or full


def slice_dir():
    """The slice's cgroup directory: from our own cgroup when we run inside it, else the user manager's."""
    try:
        own = open("/proc/self/cgroup").read().strip().split("::", 1)[1]
    except (OSError, IndexError):
        own = ""
    if f"/{SLICE}" in own:
        return "/sys/fs/cgroup" + own[:own.index(f"/{SLICE}")] + f"/{SLICE}"
    uid = os.getuid()
    return f"/sys/fs/cgroup/user.slice/user-{uid}.slice/user@{uid}.service/{SLICE}"


def slice_pids(d):
    pids = set()
    for dirpath, _, files in os.walk(d):
        if "cgroup.procs" in files:
            try:
                pids |= {int(x) for x in open(os.path.join(dirpath, "cgroup.procs")).read().split()}
            except OSError:
                pass
    return pids


def is_pcsx2(pid):
    try:
        argv0 = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")[0].decode(errors="replace")
    except OSError:
        return False
    return "pcsx2" in os.path.basename(argv0).lower()


def apply(d, mask, full):
    """Set the affinity of every thread in the slice; PCSX2 always gets `full`. Returns (threads set, pids)."""
    n = 0
    pids = slice_pids(d)
    me = os.getpid()
    for pid in pids:
        want = full if (pid == me or is_pcsx2(pid)) else mask
        try:
            tids = os.listdir(f"/proc/{pid}/task")
        except OSError:
            continue
        for tid in tids:
            try:
                if os.sched_getaffinity(int(tid)) != want:
                    os.sched_setaffinity(int(tid), want)
                    n += 1
            except (OSError, ValueError):
                pass
    return n, len(pids)


# ---------- is the maintainer here? ----------
def lid_closed():
    for f in glob.glob("/proc/acpi/button/lid/*/state"):
        try:
            if "closed" in open(f).read():
                return True
        except OSError:
            pass
    return False


_session = None


def locked():
    """logind's LockedHint of the maintainer's graphical session (KDE's screen locker sets it)."""
    global _session
    try:
        if _session is None:
            _session = os.environ.get("XDG_SESSION_ID") or ""
            if not _session:
                user = os.environ.get("USER") or ""
                for line in subprocess.run(["loginctl", "list-sessions", "--no-legend"], capture_output=True, text=True,
                                           timeout=5).stdout.splitlines():
                    p = line.split()
                    if len(p) >= 3 and p[2] == user:
                        t = subprocess.run(["loginctl", "show-session", p[0], "-p", "Type", "--value"], capture_output=True,
                                           text=True, timeout=5).stdout.strip()
                        if t in ("wayland", "x11"):
                            _session = p[0]
                            break
        if not _session:
            return False
        r = subprocess.run(["loginctl", "show-session", _session, "-p", "LockedHint", "--value"], capture_output=True,
                           text=True, timeout=5)
        return r.stdout.strip() == "yes"
    except (OSError, subprocess.SubprocessError):
        return False


INPUT_RE = re.compile(r"i8042|SYNA|ELAN|ALPS|TPPS|FTE|touchpad|trackpoint", re.I)


def input_counts():
    """(sum of the keyboard / touchpad lines, sum of the xhci lines) from /proc/interrupts; None if unreadable."""
    try:
        lines = open("/proc/interrupts").read().splitlines()
    except OSError:
        return None
    direct = usb = 0
    for l in lines[1:]:
        p = l.split()
        nums = [int(x) for x in p[1:] if x.isdigit()]
        name = " ".join(x for x in p[1:] if not x.isdigit())
        if INPUT_RE.search(name):
            direct += sum(nums)
        elif "xhci" in name:
            usb += sum(nums)
    return direct, usb


class Presence:
    def __init__(self, idle_secs):
        self.idle_secs = idle_secs
        self.last_input = time.time()      # start polite: the maintainer has just started the loop
        self.prev = input_counts()
        self.prev_t = time.time()

    def poll(self):
        """('polite'|'full', why)."""
        now = time.time()
        cur = input_counts()
        if cur is None or self.prev is None:
            self.last_input = now          # cannot read: he counts as here
        else:
            dt = max(now - self.prev_t, 1e-3)
            if cur[0] != self.prev[0] or (cur[1] - self.prev[1]) / dt > XHCI_RATE:
                self.last_input = now
        self.prev, self.prev_t = cur, now
        if lid_closed():
            return "full", "lid closed"
        if locked():
            return "full", "screen locked"
        idle = now - self.last_input
        if idle >= self.idle_secs:
            return "full", f"no input for {int(idle)} s"
        return "polite", f"input {int(idle)} s ago"


def loop_alive():
    for p in glob.glob("/proc/[0-9]*/cmdline"):
        try:
            if b"scripts/autoloop.sh" in open(p, "rb").read():
                return True
        except OSError:
            pass
    return False


def fmt(cpus):
    s = sorted(cpus)
    return f"{s[0]}-{s[-1]}" if s and s == list(range(s[0], s[-1] + 1)) else ",".join(map(str, s))


def write_mode(mode, mask, why):
    try:
        open(MODE_FILE, "w").write(f"{mode} (cpus {fmt(mask)}) since {time.strftime('%F %T')}: {why}\n")
    except OSError:
        pass


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "status"
    if cmd == "status":
        try:
            print("cpu:", open(MODE_FILE).read().strip())
        except OSError:
            print("cpu: no governor has run")
        return 0
    full, pol = all_cpus(), polite_cpus()
    d = slice_dir()
    idle_secs = float(os.environ.get("OPENBF_IDLE_SECS", "300"))
    if os.environ.get("OPENBF_POLITE", "1") == "0":
        print("loop_cpu: OPENBF_POLITE=0 - affinity left alone (the slice's CPU weight still applies)", flush=True)
        write_mode("off", full, "OPENBF_POLITE=0")
        return 0
    pres = Presence(idle_secs)
    if cmd == "once":
        mode, why = pres.poll()
        mode = os.environ.get("OPENBF_FORCE_MODE", mode)
        mask = full if mode == "full" else pol
        n, pids = apply(d, mask, full)
        print(f"{mode} ({why}): cpus {fmt(mask)}; {pids} processes in {SLICE}, {n} threads changed")
        return 0
    if cmd != "watch":
        sys.exit(__doc__)
    print(f"[loop_cpu {time.strftime('%F %T')}] watching {d}: polite = cpus {fmt(pol)}, full = {fmt(full)}, idle after "
          f"{int(idle_secs)} s", flush=True)
    last, dead = None, 0
    while True:
        mode, why = pres.poll()
        mask = full if mode == "full" else pol
        apply(d, mask, full)
        if mode != last:
            print(f"[loop_cpu {time.strftime('%F %T')}] {mode}: cpus {fmt(mask)} ({why})", flush=True)
            write_mode(mode, mask, why)
            last = mode
        dead = 0 if loop_alive() else dead + 1
        if dead >= 12:      # a minute with no autoloop.sh (it is started just after us)
            print(f"[loop_cpu {time.strftime('%F %T')}] no autoloop.sh left; exiting", flush=True)
            write_mode("stopped", full, "the loop is not running")
            return 0
        time.sleep(POLL)


if __name__ == "__main__":
    sys.exit(main())
