#!/usr/bin/env python3
"""Pad a Mach-O LINKEDIT string pool so its file offset is 8-byte aligned.

dyld on this OS refuses to load an arm64 image whose LC_SYMTAB.stroff is not a
multiple of 8 (``mis-aligned LINKEDIT string pool``). The Xcode 27 linker emits
that offset directly after an odd-length indirect-symbol table. ``-ld_classic``
is ignored by this ld, so the cdylib is padded after the link.

The string index of every symbol is relative to stroff, so inserting the pad
*before* the pool and advancing every later linkedit offset keeps the table valid.
"""

from __future__ import annotations

import struct
import subprocess
import sys

MH_MAGIC_64 = 0xFEEDFACF
LC_SYMTAB = 0x2
LC_SEGMENT_64 = 0x19
LC_DYLD_INFO = 0x22
LC_DYLD_INFO_ONLY = 0x80000022
# linkedit_data_command: cmd, cmdsize, dataoff, datasize
LINKEDIT_DATA = {
    0x1D,  # LC_CODE_SIGNATURE
    0x1E,  # LC_SEGMENT_SPLIT_INFO
    0x26,  # LC_FUNCTION_STARTS
    0x29,  # LC_DATA_IN_CODE
    0x2B,  # LC_DYLIB_CODE_SIGN_DRS
    0x2E,  # LC_LINKER_OPTIMIZATION_HINT
    0x80000033,  # LC_DYLD_EXPORTS_TRIE
    0x80000034,  # LC_DYLD_CHAINED_FIXUPS
}
PAGE = 16384


def pad_file(path: str) -> bool:
    data = bytearray(open(path, "rb").read())
    if len(data) < 32:
        return False
    magic = struct.unpack_from("<I", data, 0)[0]
    if magic != MH_MAGIC_64:
        return False
    ncmds = struct.unpack_from("<I", data, 16)[0]
    off = 32
    stroff = None
    symtab_at = None
    linkedit = None
    dyld_info_at = []
    data_cmds = []
    for _ in range(ncmds):
        cmd, cmdsize = struct.unpack_from("<II", data, off)
        if cmd == LC_SYMTAB:
            stroff = struct.unpack_from("<I", data, off + 16)[0]
            symtab_at = off
        elif cmd == LC_SEGMENT_64:
            name = bytes(data[off + 8 : off + 24]).split(b"\0", 1)[0]
            if name == b"__LINKEDIT":
                vmsize = struct.unpack_from("<Q", data, off + 32)[0]
                fileoff, filesize = struct.unpack_from("<QQ", data, off + 40)
                linkedit = (off, vmsize, fileoff, filesize)
        elif cmd in (LC_DYLD_INFO, LC_DYLD_INFO_ONLY):
            dyld_info_at.append(off)
        elif cmd in LINKEDIT_DATA:
            data_cmds.append(off)
        off += cmdsize
    if stroff is None or linkedit is None or stroff % 8 == 0:
        return False
    # The string pool needs 8-byte alignment, and so does every later blob
    # (the code signature in particular). One insert would fix the pool and
    # break the signature, so a second insert in front of the signature puts
    # that blob back on an 8-byte boundary.
    pad_str = 8 - (stroff % 8)
    sig_off = None
    for at in data_cmds:
        cmd = struct.unpack_from("<I", data, at)[0]
        if cmd == 0x1D:
            sig_off = struct.unpack_from("<I", data, at + 8)[0]
    data[stroff:stroff] = b"\0" * pad_str
    pad_sig = 0
    if sig_off is not None and sig_off >= stroff:
        moved = sig_off + pad_str
        pad_sig = (8 - (moved % 8)) % 8
        if pad_sig:
            data[moved:moved] = b"\0" * pad_sig
    struct.pack_into("<I", data, symtab_at + 16, stroff + pad_str)

    def shift(rel: int) -> int:
        if rel >= (sig_off if sig_off is not None else 2**32):
            return rel + pad_str + pad_sig
        if rel >= stroff:
            return rel + pad_str
        return rel

    seg_at, vmsize, _fileoff, filesize = linkedit
    new_size = filesize + pad_str + pad_sig
    struct.pack_into("<Q", data, seg_at + 48, new_size)
    if vmsize < new_size:
        new_vm = (new_size + PAGE - 1) & ~(PAGE - 1)
        struct.pack_into("<Q", data, seg_at + 32, new_vm)
    for at in data_cmds:
        dataoff = struct.unpack_from("<I", data, at + 8)[0]
        struct.pack_into("<I", data, at + 8, shift(dataoff))
    for at in dyld_info_at:
        for i in range(5):
            field = at + 8 + i * 8
            rel = struct.unpack_from("<I", data, field)[0]
            struct.pack_into("<I", data, field, shift(rel))
    with open(path, "wb") as fh:
        fh.write(data)
    subprocess.check_call(["codesign", "--force", "--sign", "-", path], stdout=subprocess.DEVNULL)
    return True


def main() -> None:
    changed = False
    for path in sys.argv[1:]:
        if pad_file(path):
            print(f"padded {path}")
            changed = True
    sys.exit(0 if changed or sys.argv[1:] else 1)


if __name__ == "__main__":
    main()
