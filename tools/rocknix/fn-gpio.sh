#!/bin/sh
# Reads the GPIO pin that older ROCKNIX used for the RGB20SX FN button (gpio3 PB7,
# active low), to see whether FN shows up there. The pin is requested as an input for a
# few seconds and released again. Nothing is stopped or changed. Run over SSH as root:
#
#   sh fn-gpio.sh
#
# Output is printed and saved to /storage/slot-fn-gpio.txt.

OUT="${PROBE_OUT:-/storage/slot-fn-gpio.txt}"

say() {
	printf '%s\n' "$*" > /dev/tty
}

{
	echo "slot FN GPIO check, $(date)"
	echo
	echo "===== tools"
	command -v gpioget gpioinfo || echo "no libgpiod tools, using python3"
	gpioinfo gpiochip3 2>/dev/null | sed -n '1p;17p'
	echo
	echo "===== gpio3 PB7 (bank gpio3, line 15)"
	python3 - <<'EOF'
import fcntl, glob, os, struct, time

LINE = 15  # PB7: bank B is lines 8-15
CHIPINFO = 0x8044B401
GET_LINE = 0xC250B407
GET_VALUES = 0xC010B40E
FLAG_INPUT = 1 << 2
BIAS = {"pull-up": 1 << 8, "pull-down": 1 << 9, "as-is": 0}

def tty(msg):
    with open("/dev/tty", "w") as t:
        t.write(msg + "\n")

def find_chip():
    for path in sorted(glob.glob("/dev/gpiochip*")):
        fd = os.open(path, os.O_RDWR)
        info = bytearray(68)
        fcntl.ioctl(fd, CHIPINFO, info)
        label = info[32:64].split(b"\0")[0].decode()
        name = info[0:32].split(b"\0")[0].decode()
        lines = struct.unpack_from("<I", info, 64)[0]
        if label == "gpio3":
            print("chip: %s (%s, label %s, %d lines)" % (path, name, label, lines))
            return fd
        os.close(fd)
    return None

def request(chip, bias):
    req = bytearray(592)
    struct.pack_into("<I", req, 0, LINE)
    req[256:256 + 8] = b"slot-fn\0"
    struct.pack_into("<Q", req, 288, FLAG_INPUT | BIAS[bias])
    struct.pack_into("<I", req, 560, 1)
    fcntl.ioctl(chip, GET_LINE, req)
    return struct.unpack_from("<i", req, 588)[0]

def read(line_fd):
    vals = bytearray(16)
    struct.pack_into("<QQ", vals, 0, 0, 1)
    fcntl.ioctl(line_fd, GET_VALUES, vals)
    return struct.unpack_from("<Q", vals, 0)[0] & 1

def sample(line_fd, seconds):
    seen = []
    end = time.time() + seconds
    while time.time() < end:
        seen.append(read(line_fd))
        time.sleep(0.05)
    return "%d samples, high %d, low %d" % (len(seen), seen.count(1), seen.count(0))

chip = find_chip()
if chip is None:
    print("no gpiochip labelled gpio3")
    raise SystemExit
for bias in ("as-is", "pull-up"):
    try:
        line = request(chip, bias)
    except OSError as e:
        print("bias %s: request refused: %s" % (bias, os.strerror(e.errno)))
        continue
    tty("[%s] Leave FN alone for 2 seconds..." % bias)
    time.sleep(0.3)
    print("bias %s, FN released: %s" % (bias, sample(line, 2)))
    tty("[%s] Now HOLD FN down (3 seconds)..." % bias)
    time.sleep(0.5)
    print("bias %s, FN held: %s" % (bias, sample(line, 2.5)))
    tty("[%s] Let go." % bias)
    os.close(line)
    time.sleep(1)
EOF
	echo
	echo "===== done"
} 2>&1 | tee "$OUT"
say "Saved to $OUT"
