"""Compute the largest centred square that can be cleared from a QR code so it still decodes.

For every version (1-40) and error correction level, this works out from ISO/IEC 18004 alone
(no encoder, no decoder) which codewords a centred s x s square touches, and keeps the largest odd
s for which every Reed-Solomon block can still correct all of them:

1. Placement. Codeword placement depends only on the version, not on the data or the mask, so the
   module -> codeword map is built once per version, skipping function patterns.
2. Damage. Every codeword with at least one module inside the square is assumed to be wrong. This
   is the worst case, so the result holds for any payload, mask and graphic.
3. Safety. The square is safe if, for every block, the touched codewords fit within that block's
   correction capacity t = (EC codewords - p) / 2, where p is the number of codewords the standard
   reserves to detect misdecodes in the smallest symbols, minus a margin kept for real-world
   errors such as print defects and blur.
4. Function patterns are located geometrically, not repaired by error correction, so the square
   must not touch finder patterns, separators, timing patterns, format or version information, or
   the dark module.

Alignment patterns may be covered. In versions 7-13, 21-27 and 35-40 one sits at or right next to
the centre, so avoiding them would rule out a graphic altogether, and decoders find the grid
without them: ZXing and quirc only search for the one nearest the bottom right corner.

The margin defaults to 2 codewords per block. With 0.1% of modules flipped at random on top of the
graphic, rxing decoded 70% of codes at margin 0, 94.5% at 1, 99.3% at 2 and 100% at 3 (level H, all
versions, `decode-check --noise 0.001`).

Usage: python tools/safe-sizes.py [--margin N] [--output FILE]
Without --output the table is printed instead of written.
"""

import argparse
import sys
from pathlib import Path

VERSION = "0.1.0"
LEVELS = "lmqh"
MARGIN = 2

# ISO/IEC 18004:2015 Table 9: per version, for levels L, M, Q, H:
# (EC codewords per block, blocks in group 1, data codewords per group 1 block, blocks in group 2)
# Group 2 blocks have one more data codeword than group 1 blocks.
EC_BLOCKS = [
    None,
    ((7, 1, 19, 0), (10, 1, 16, 0), (13, 1, 13, 0), (17, 1, 9, 0)),
    ((10, 1, 34, 0), (16, 1, 28, 0), (22, 1, 22, 0), (28, 1, 16, 0)),
    ((15, 1, 55, 0), (26, 1, 44, 0), (18, 2, 17, 0), (22, 2, 13, 0)),
    ((20, 1, 80, 0), (18, 2, 32, 0), (26, 2, 24, 0), (16, 4, 9, 0)),
    ((26, 1, 108, 0), (24, 2, 43, 0), (18, 2, 15, 2), (22, 2, 11, 2)),
    ((18, 2, 68, 0), (16, 4, 27, 0), (24, 4, 19, 0), (28, 4, 15, 0)),
    ((20, 2, 78, 0), (18, 4, 31, 0), (18, 2, 14, 4), (26, 4, 13, 1)),
    ((24, 2, 97, 0), (22, 2, 38, 2), (22, 4, 18, 2), (26, 4, 14, 2)),
    ((30, 2, 116, 0), (22, 3, 36, 2), (20, 4, 16, 4), (24, 4, 12, 4)),
    ((18, 2, 68, 2), (26, 4, 43, 1), (24, 6, 19, 2), (28, 6, 15, 2)),
    ((20, 4, 81, 0), (30, 1, 50, 4), (28, 4, 22, 4), (24, 3, 12, 8)),
    ((24, 2, 92, 2), (22, 6, 36, 2), (26, 4, 20, 6), (28, 7, 14, 4)),
    ((26, 4, 107, 0), (22, 8, 37, 1), (24, 8, 20, 4), (22, 12, 11, 4)),
    ((30, 3, 115, 1), (24, 4, 40, 5), (20, 11, 16, 5), (24, 11, 12, 5)),
    ((22, 5, 87, 1), (24, 5, 41, 5), (30, 5, 24, 7), (24, 11, 12, 7)),
    ((24, 5, 98, 1), (28, 7, 45, 3), (24, 15, 19, 2), (30, 3, 15, 13)),
    ((28, 1, 107, 5), (28, 10, 46, 1), (28, 1, 22, 15), (28, 2, 14, 17)),
    ((30, 5, 120, 1), (26, 9, 43, 4), (28, 17, 22, 1), (28, 2, 14, 19)),
    ((28, 3, 113, 4), (26, 3, 44, 11), (26, 17, 21, 4), (26, 9, 13, 16)),
    ((28, 3, 107, 5), (26, 3, 41, 13), (30, 15, 24, 5), (28, 15, 15, 10)),
    ((28, 4, 116, 4), (26, 17, 42, 0), (28, 17, 22, 6), (30, 19, 16, 6)),
    ((28, 2, 111, 7), (28, 17, 46, 0), (30, 7, 24, 16), (24, 34, 13, 0)),
    ((30, 4, 121, 5), (28, 4, 47, 14), (30, 11, 24, 14), (30, 16, 15, 14)),
    ((30, 6, 117, 4), (28, 6, 45, 14), (30, 11, 24, 16), (30, 30, 16, 2)),
    ((26, 8, 106, 4), (28, 8, 47, 13), (30, 7, 24, 22), (30, 22, 15, 13)),
    ((28, 10, 114, 2), (28, 19, 46, 4), (28, 28, 22, 6), (30, 33, 16, 4)),
    ((30, 8, 122, 4), (28, 22, 45, 3), (30, 8, 23, 26), (30, 12, 15, 28)),
    ((30, 3, 117, 10), (28, 3, 45, 23), (30, 4, 24, 31), (30, 11, 15, 31)),
    ((30, 7, 116, 7), (28, 21, 45, 7), (30, 1, 23, 37), (30, 19, 15, 26)),
    ((30, 5, 115, 10), (28, 19, 47, 10), (30, 15, 24, 25), (30, 23, 15, 25)),
    ((30, 13, 115, 3), (28, 2, 46, 29), (30, 42, 24, 1), (30, 23, 15, 28)),
    ((30, 17, 115, 0), (28, 10, 46, 23), (30, 10, 24, 35), (30, 19, 15, 35)),
    ((30, 17, 115, 1), (28, 14, 46, 21), (30, 29, 24, 19), (30, 11, 15, 46)),
    ((30, 13, 115, 6), (28, 14, 46, 23), (30, 44, 24, 7), (30, 59, 16, 1)),
    ((30, 12, 121, 7), (28, 12, 47, 26), (30, 39, 24, 14), (30, 22, 15, 41)),
    ((30, 6, 121, 14), (28, 6, 47, 34), (30, 46, 24, 10), (30, 2, 15, 64)),
    ((30, 17, 122, 4), (28, 29, 46, 14), (30, 49, 24, 10), (30, 24, 15, 46)),
    ((30, 4, 122, 18), (28, 13, 46, 32), (30, 48, 24, 14), (30, 42, 15, 32)),
    ((30, 20, 117, 4), (28, 40, 47, 7), (30, 43, 24, 22), (30, 10, 15, 67)),
    ((30, 19, 118, 6), (28, 18, 47, 31), (30, 34, 24, 34), (30, 20, 15, 61)),
]

# ISO/IEC 18004:2015 Table 9 note: misdecode protection codewords p, where t = (EC - p) / 2
MISDECODE_PROTECTION = {
    (1, "l"): 3,
    (1, "m"): 2,
    (1, "q"): 1,
    (1, "h"): 1,
    (2, "l"): 2,
    (3, "l"): 1,
}

# fmt: off
# ISO/IEC 18004:2015 Table 1, used as a cross-check of the derived codeword counts
TOTAL_CODEWORDS = [
    None, 26, 44, 70, 100, 134, 172, 196, 242, 292, 346, 404, 466, 532, 581, 655, 733, 815, 901,
    991, 1085, 1156, 1258, 1364, 1474, 1588, 1706, 1828, 1921, 2051, 2185, 2323, 2465, 2611, 2761,
    2876, 3034, 3196, 3362, 3532, 3706,
]

# ISO/IEC 18004:2015 Annex E: alignment pattern centre coordinates
ALIGNMENT_POSITIONS = [
    None, (), (6, 18), (6, 22), (6, 26), (6, 30), (6, 34),
    (6, 22, 38), (6, 24, 42), (6, 26, 46), (6, 28, 50), (6, 30, 54), (6, 32, 58), (6, 34, 62),
    (6, 26, 46, 66), (6, 26, 48, 70), (6, 26, 50, 74), (6, 30, 54, 78), (6, 30, 56, 82),
    (6, 30, 58, 86), (6, 34, 62, 90),
    (6, 28, 50, 72, 94), (6, 26, 50, 74, 98), (6, 30, 54, 78, 102), (6, 28, 54, 80, 106),
    (6, 32, 58, 84, 110), (6, 30, 58, 86, 114), (6, 34, 62, 90, 118),
    (6, 26, 50, 74, 98, 122), (6, 30, 54, 78, 102, 126), (6, 26, 52, 78, 104, 130),
    (6, 30, 56, 82, 108, 134), (6, 34, 60, 86, 112, 138), (6, 30, 58, 86, 114, 142),
    (6, 34, 62, 90, 118, 146),
    (6, 30, 54, 78, 102, 126, 150), (6, 24, 50, 76, 102, 128, 154), (6, 28, 54, 80, 106, 132, 158),
    (6, 32, 58, 84, 110, 136, 162), (6, 26, 54, 82, 110, 138, 166), (6, 30, 58, 86, 114, 142, 170),
]
# fmt: on

ALIGNMENT = "alignment"


def symbol_size(version):
    return 17 + 4 * version


def function_modules(version):
    """Map of (x, y) -> kind for every function module."""
    n = symbol_size(version)
    kinds = {}

    def mark(xs, ys, kind):
        for y in ys:
            for x in xs:
                kinds.setdefault((x, y), kind)

    # finder patterns with their separators
    for x0, y0 in ((0, 0), (n - 8, 0), (0, n - 8)):
        mark(range(x0, x0 + 8), range(y0, y0 + 8), "finder")
    # format information (both copies) and the dark module
    mark([8], range(9), "format")
    mark(range(9), [8], "format")
    mark(range(n - 8, n), [8], "format")
    mark([8], range(n - 8, n), "format")  # (8, n - 8) is the dark module
    mark([6], range(n), "timing")
    mark(range(n), [6], "timing")
    # alignment patterns, except the three that would overlap finder patterns
    positions = ALIGNMENT_POSITIONS[version]
    for cy in positions:
        for cx in positions:
            if (cx, cy) in ((6, 6), (6, positions[-1]), (positions[-1], 6)):
                continue
            mark(range(cx - 2, cx + 3), range(cy - 2, cy + 3), ALIGNMENT)
    if version >= 7:
        mark(range(n - 11, n - 8), range(6), "version")
        mark(range(6), range(n - 11, n - 8), "version")
    return kinds


def placement_order(version, functions):
    """Data modules in the order codeword bits are placed (ISO/IEC 18004:2015 7.7.3)."""
    n = symbol_size(version)
    order = []
    right = n - 1
    while right >= 1:
        if right == 6:  # the vertical timing pattern is skipped as a whole column
            right = 5
        upward = (right + 1) & 2 == 0
        for i in range(n):
            y = n - 1 - i if upward else i
            for x in (right, right - 1):
                if (x, y) not in functions:
                    order.append((x, y))
        right -= 2
    return order


def blocks(version, level):
    """(data codewords, EC codewords) for each block, in interleaving order."""
    ec, count1, data1, count2 = EC_BLOCKS[version][LEVELS.index(level)]
    return [(data1, ec)] * count1 + [(data1 + 1, ec)] * count2


def codeword_blocks(version, level):
    """Block index of each codeword in the final, interleaved sequence (ISO/IEC 18004:2015 7.6)."""
    layout = blocks(version, level)
    owners = []
    for i in range(max(data for data, _ in layout)):
        owners += [b for b, (data, _) in enumerate(layout) if i < data]
    for i in range(layout[0][1]):
        owners += list(range(len(layout)))
    return owners


def capacity(version, level):
    """Errors each block can correct according to the standard."""
    _, ec = blocks(version, level)[0]
    return (ec - MISDECODE_PROTECTION.get((version, level), 0)) // 2


def check_tables(version, order):
    """Cross-check Table 9 against the codeword count derived from the module layout."""
    total = len(order) // 8
    assert total == TOTAL_CODEWORDS[version], (version, total)
    assert len(order) % 8 in (0, 3, 4, 7), version
    for level in LEVELS:
        assert sum(d + e for d, e in blocks(version, level)) == total, (version, level)


def max_safe_size(version, level, margin, functions, order):
    """Largest odd side of a centred square that can be cleared, or 0 if there is none."""
    n = symbol_size(version)
    owners = codeword_blocks(version, level)
    bit_of = {module: i for i, module in enumerate(order)}
    allowed = capacity(version, level) - margin

    best = 0
    for side in range(1, n + 1, 2):
        start = (n - side) // 2
        square = [
            (x, y)
            for y in range(start, start + side)
            for x in range(start, start + side)
        ]
        if any(functions.get(m, ALIGNMENT) != ALIGNMENT for m in square):
            break
        touched = {
            bit_of[m] // 8
            for m in square
            if m in bit_of and bit_of[m] // 8 < len(owners)
        }
        counts = [0] * (max(owners) + 1)
        for codeword in touched:
            counts[owners[codeword]] += 1
        if max(counts) > allowed:
            break
        best = side
    return best


def compute(margin):
    table = {level: [] for level in LEVELS}
    for version in range(1, 41):
        functions = function_modules(version)
        order = placement_order(version, functions)
        check_tables(version, order)
        for level in LEVELS:
            table[level].append(max_safe_size(version, level, margin, functions, order))
    return table


def render(table, margin):
    lines = [
        "# Largest centred square, as a side length in modules, that can be cleared from a QR code",
        "# and still decode, for versions 1-40 at each error correction level. 0 means none.",
        "# Generated by tools/safe-sizes.py from ISO/IEC 18004. Do not edit; run `just sizes`.",
        "",
        f'generator = "safe-sizes.py {VERSION}"',
        f"margin = {margin}  # codewords per block kept in reserve",
        "",
    ]
    for level in LEVELS:
        values = table[level]
        rows = [
            ", ".join(f"{v:3}" for v in values[i : i + 10]) for i in range(0, 40, 10)
        ]
        lines.append(f"{level} = [\n  " + ",\n  ".join(rows) + ",\n]")
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "--margin",
        type=int,
        default=MARGIN,
        help=f"codewords per block to keep in reserve (default {MARGIN})",
    )
    parser.add_argument("--output", type=Path, help="file to write the table to")
    args = parser.parse_args()

    text = render(compute(args.margin), args.margin)
    if args.output:
        args.output.write_text(text, encoding="utf-8", newline="\n")
        print(f"Wrote {args.output}", file=sys.stderr)
    else:
        print(text, end="")


if __name__ == "__main__":
    main()
