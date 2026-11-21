"""Decode SCS binary BSII (versions 1-3) into text SiiNunit. Usage: bsii_decode.py IN OUT"""
import struct, sys

CHARS = "0123456789abcdefghijklmnopqrstuvwxyz_"


class R:
    def __init__(self, b):
        self.b, self.p = b, 0

    def take(self, n):
        v = self.b[self.p:self.p + n]
        if len(v) != n:
            raise EOFError(self.p)
        self.p += n
        return v

    def u(self, fmt):
        return struct.unpack("<" + fmt, self.take(struct.calcsize("<" + fmt)))

    def u8(self): return self.u("B")[0]
    def u32(self): return self.u("I")[0]
    def u64(self): return self.u("Q")[0]
    def s(self): return self.take(self.u32()).decode("utf-8", "replace")


def enc_str(v):
    out = []
    while v:
        v, i = divmod(v, 38)
        out.append(CHARS[i - 1])
    return "".join(out)


def fmt_f(f):
    if f == int(f) and abs(f) < 1e9:
        return str(int(f))
    return repr(round(f, 6))


def q(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def read_id(r):
    n = r.u8()
    if n == 0xFF:
        v = r.u64()
        parts = [f"{(v >> s) & 0xFFFF:x}" for s in (48, 32, 16, 0)]
        while len(parts) > 1 and parts[0] == "0":
            parts.pop(0)
        return "_nameless." + ".".join(parts)
    if n == 0:
        return "null"
    return ".".join(enc_str(r.u64()) for _ in range(n))


def plain_token(s):
    return s if s and all(c in CHARS for c in s) else q(s)


def make_readers(version):
    f = lambda r: fmt_f(r.u("f")[0])
    vec = lambda n: (lambda r: "(" + ", ".join(fmt_f(x) for x in r.u("f" * n)) + ")")

    def vec8(r):
        # Version 1 stores 7 floats; version 2+ adds a hidden bias (4th float) for x and z.
        if version == 1:
            v = list(r.u("7f"))
            return "(" + ", ".join(fmt_f(x) for x in v[:3]) + ") (" + ", ".join(fmt_f(x) for x in v[3:]) + ")"
        v = list(r.u("8f"))
        if version >= 2:
            bias = int(v[3])
            v[0] += ((bias & 0xFFF) - 2048) << 9
            v[2] += (((bias >> 12) & 0xFFF) - 2048) << 9
        return "(" + ", ".join(fmt_f(x) for x in v[:3]) + ") (" + ", ".join(fmt_f(x) for x in v[4:]) + ")"

    single = {
        0x01: lambda r: q(r.s()),
        0x03: lambda r: enc_str(r.u64()) or '""',
        0x05: f,
        0x07: vec(2),
        0x09: vec(3),
        0x11: lambda r: "(" + ", ".join(map(str, r.u("3i"))) + ")",
        0x17: vec(4),
        0x19: vec8,
        0x25: lambda r: str(r.u("i")[0]),
        0x27: lambda r: str(r.u32()),
        0x29: lambda r: str(r.u("h")[0]),
        0x2B: lambda r: str(r.u("H")[0]),
        0x2F: lambda r: str(r.u32()),
        0x31: lambda r: str(r.u("q")[0]),
        0x33: lambda r: str(r.u64()),
        0x35: lambda r: "true" if r.u8() else "false",
        0x39: read_id, 0x3B: read_id, 0x3D: read_id,
    }
    arrays = {k + 1: v for k, v in single.items() if k not in (0x2F, 0x35, 0x39, 0x3B, 0x3D)}
    arrays.update({0x36: single[0x35], 0x3A: read_id, 0x3C: read_id, 0x3E: read_id})
    return single, arrays


def decode(b):
    r = R(b)
    assert r.take(4) == b"BSII"
    version = r.u32()
    single, arrays = make_readers(version)
    structs = {}
    out = ["SiiNunit", "{"]
    while r.p < len(b):
        btype = r.u32()
        if btype == 0:
            if not r.u8():
                break
            sid, name = r.u32(), r.s()
            fields = []
            while True:
                t = r.u32()
                if t == 0:
                    break
                fname = r.s()
                table = None
                if t == 0x37:
                    table = {}
                    for _ in range(r.u32()):
                        idx = r.u32()
                        table[idx] = r.s()
                fields.append((t, fname, table))
            structs[sid] = (name, fields)
            continue
        name, fields = structs[btype]
        out.append(f"{name} : {read_id(r)} {{")
        for t, fname, table in fields:
            if t == 0x37:
                out.append(f" {fname}: {plain_token(table.get(r.u32(), ''))}")
            elif t in single:
                out.append(f" {fname}: {single[t](r)}")
            elif t in arrays:
                n = r.u32()
                out.append(f" {fname}: {n}")
                for i in range(n):
                    out.append(f" {fname}[{i}]: {arrays[t](r)}")
            else:
                raise ValueError(f"unknown type 0x{t:x} for {name}.{fname} at {r.p}")
        out.append("}")
        out.append("")
    out.append("}")
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    txt = decode(open(sys.argv[1], "rb").read())
    open(sys.argv[2], "w", encoding="utf-8").write(txt)
    print(sys.argv[2], len(txt.splitlines()), "lines")
