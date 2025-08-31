#!/usr/bin/env python3
"""
visualize_udp_stats.py

Reads UDP throughput logs produced by your server.py (and/or the Rust client)
and produces a clean plot + CSV of per-second KB/s.

Usage examples:
  # Read from a saved log file and show the plot
  python3 visualize_udp_stats.py --input server_run.log

  # Live: pipe server output into the visualizer
  python3 server.py | python3 visualize_udp_stats.py --from-stdin

  # Save outputs to a specific prefix
  python3 visualize_udp_stats.py --input server_run.log --out-prefix results/test1

This script understands lines like:
  "0 - 1: 123.4 KB/s"                       (server.py format)
  "[0 - 1] : [123.4 KB/s]"                 (Rust client format)

It will also pick up the final per-second line before "Received exit: End reception".
"""

import argparse
import re
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import csv

# Regexes to catch both server.py and Rust client's interval lines
RE_SERVER = re.compile(r'^\s*(\d+)\s*-\s*(\d+)\s*:\s*([0-9.]+)\s*KB/s\s*$')
RE_CLIENT = re.compile(r'^\s*\[\s*(\d+)\s*-\s*(\d+)\s*\]\s*:\s*\[\s*([0-9.]+)\s*KB/s\s*\]\s*$')

def parse_lines(iter_lines):
    """
    Return a list of (second_index, kbps_float) extracted from the given lines.
    second_index uses the right-hand interval bound, e.g., for "0-1" we store 1.
    """
    points = []
    for line in iter_lines:
        line = line.strip()
        if not line:
            continue
        m = RE_SERVER.match(line)
        if not m:
            m = RE_CLIENT.match(line)
        if m:
            _l, r, val = m.groups()
            try:
                sec = int(r)
                kbps = float(val)
                points.append((sec, kbps))
            except ValueError:
                pass
    # Deduplicate by sec (keep last for that sec)
    latest = {}
    for sec, kbps in points:
        latest[sec] = kbps
    points = sorted(latest.items(), key=lambda x: x[0])
    return points

def write_csv(points, csv_path: Path):
    csv_path.parent.mkdir(parents=True, exist_ok=True)
    with csv_path.open('w', newline='') as f:
        w = csv.writer(f)
        w.writerow(['second', 'KB_per_s'])
        for sec, kbps in points:
            w.writerow([sec, kbps])

def make_plot(points, png_path: Path, title: str):
    png_path.parent.mkdir(parents=True, exist_ok=True)
    secs = [sec for sec, _ in points]
    kbps = [v for _, v in points]

    plt.figure()
    plt.plot(secs, kbps, marker='o')
    plt.xlabel('Second')
    plt.ylabel('KB/s')
    plt.title(title)
    plt.grid(True)
    plt.tight_layout()
    plt.savefig(png_path)
    # Also show interactively if running in a desktop
    try:
        plt.show()
    except Exception:
        pass

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--input', '-i', type=str, help='Path to a saved log file. If omitted, requires --from-stdin.')
    ap.add_argument('--from-stdin', action='store_true', help='Read log lines from stdin.')
    ap.add_argument('--out-prefix', type=str, default='udp_visualization', help='Prefix for output files (CSV/PNG).')
    args = ap.parse_args()

    if not args.input and not args.from_stdin:
        print('Error: provide --input FILE or --from-stdin', file=sys.stderr)
        sys.exit(2)

    lines_iter = None
    title_bits = []
    if args.input:
        p = Path(args.input)
        if not p.exists():
            print(f'Error: file not found: {p}', file=sys.stderr)
            sys.exit(2)
        lines_iter = p.open('r', errors='ignore')
        title_bits.append(str(p))
    if args.from_stdin:
        lines_iter = sys.stdin if lines_iter is None else lines_iter
        title_bits.append('stdin')

    points = parse_lines(lines_iter)
    if hasattr(lines_iter, 'close'):
        try:
            lines_iter.close()
        except Exception:
            pass

    if not points:
        print('No per-second KB/s lines found. Make sure your server/client are printing lines like "0 - 1: 123 KB/s".', file=sys.stderr)
        sys.exit(1)

    out_prefix = Path(args.out_prefix)
    csv_path = out_prefix.with_suffix('.csv')
    png_path = out_prefix.with_suffix('.png')

    write_csv(points, csv_path)
    title = 'UDP Throughput (KB/s) — ' + ' + '.join(title_bits)
    make_plot(points, png_path, title)

    print(f'Wrote CSV: {csv_path}')
    print(f'Wrote plot: {png_path}')

if __name__ == '__main__':
    main()
