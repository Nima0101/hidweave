"""Compare a deliberately shared variable-field subset with hid-parser 0.1.0.

This experiment is independent test tooling, not a runtime dependency. It does
not treat either implementation as a universal conformance oracle.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile

import hid_parser

root = Path(__file__).resolve().parents[1]
subprocess.run(["cargo", "build", "--locked", "--offline"], cwd=root, check=True)
binary = root / "target" / "debug" / ("hidweave.exe" if os.name == "nt" else "hidweave")
samples = 0
with tempfile.TemporaryDirectory() as temp:
    path = Path(temp) / "descriptor.bin"
    wire_path = Path(temp) / "report.bin"
    # Numeric usages X/Y; an independent parser supplies offsets and values.
    # Bit widths remain 1..7 so the oracle's signedness behavior is unambiguous.
    for width in range(1, 8):
        for padding in range(8):
            for signed in (False, True):
                data = bytearray.fromhex("05 01 09 04 a1 01")
                if padding:
                    data.extend([0x75, padding, 0x95, 1, 0x81, 1])
                minimum = -(1 << (width - 1)) if signed else 0
                maximum = (1 << (width - int(signed))) - 1
                data.extend([0x15, minimum & 255, 0x25, maximum, 0x75, width, 0x95, 2, 0x09, 0x30, 0x09, 0x31, 0x81, 2, 0xc0])
                path.write_bytes(data)
                oracle = hid_parser.ReportDescriptor(data)
                ours = json.loads(subprocess.check_output([str(binary), "inspect", str(path), "--json"]))["reports"][0]
                items = [item for item in oracle.get_input_items() if isinstance(item, hid_parser.VariableItem)]
                fields = [field for field in ours["fields"] if field["meaning"]["type"] == "variable"]
                assert len(items) == len(fields) == 2
                for item, field in zip(items, fields):
                    assert int(item.offset) == field["bit_offset"]
                    assert int(item.size) == field["bit_size"]
                    assert item.logical_min == field["semantics"]["logical_min"]
                    assert item.logical_max == field["semantics"]["logical_max"]
                samples += 1
    # Shared byte-aligned unsigned decode subset. The oracle 0.1.0 does not
    # sign-extend signed values and disagrees on sub-byte bit order; see below.
    for width in (8, 16):
        data = bytes.fromhex("05 01 09 04 a1 01 15 00 26 ff 7f 75 10 95 02 09 30 09 31 81 02 c0") if width == 16 else bytes.fromhex("05 01 09 04 a1 01 15 00 26 ff 00 75 08 95 02 09 30 09 31 81 02 c0")
        path.write_bytes(data)
        oracle = hid_parser.ReportDescriptor(data)
        for raw in (0, 1, 127):
            wire = raw.to_bytes(width // 8, "little") * 2
            wire_path.write_bytes(wire)
            values = json.loads(subprocess.check_output([str(binary), "decode", str(path), str(wire_path), "--kind", "input", "--json"]))["decoded"]["values"]
            reference = oracle.parse_input_report(wire)
            assert [reference[item.usage].value for item in oracle.get_input_items()] == [v["value"] for v in values]
    # Pin two observed disagreements explicitly; never broaden oracle claims.
    one_bit = bytes.fromhex("05 01 09 04 a1 01 15 00 25 01 75 01 95 02 09 30 09 31 81 02 c0")
    signed = bytes.fromhex("05 01 09 04 a1 01 15 81 25 7f 75 08 95 02 09 30 09 31 81 02 c0")
    for descriptor, wire, ours_expected, reference_expected in ((one_bit, b"\x03", 1, 0), (signed, b"\xff\x00", -1, 255)):
        path.write_bytes(descriptor)
        wire_path.write_bytes(wire)
        ours = json.loads(subprocess.check_output([str(binary), "decode", str(path), str(wire_path), "--kind", "input", "--json"]))["decoded"]["values"][0]["value"]
        oracle = hid_parser.ReportDescriptor(descriptor)
        reference = next(iter(oracle.parse_input_report(wire).values())).value
        assert (ours, reference) == (ours_expected, reference_expected)
print(f"Independent parser experiment: {samples} matching descriptor layouts, 6 matching byte-aligned unsigned reports; 2 documented decoder disagreements reproduced")
