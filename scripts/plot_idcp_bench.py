#!/usr/bin/env python3
import csv
import html
import sys
from pathlib import Path


def read_rows(csv_path: Path):
    with csv_path.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def to_float(row, key):
    return float(row[key])


def svg_bar_chart(labels, series, title, ylabel, output_path):
    width = 1200
    height = 700
    margin_left = 90
    margin_right = 30
    margin_top = 70
    margin_bottom = 170
    plot_w = width - margin_left - margin_right
    plot_h = height - margin_top - margin_bottom

    values = [v for _, vals, _ in series for v in vals]
    min_v = min(values + [0.0])
    max_v = max(values + [0.0])
    if max_v == min_v:
        max_v += 1.0

    def y_of(value):
        ratio = (value - min_v) / (max_v - min_v)
        return margin_top + plot_h - ratio * plot_h

    x_count = len(labels)
    group_w = plot_w / max(x_count, 1)
    bar_w = group_w / (len(series) + 1)

    lines = []
    lines.append(f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}">')
    lines.append(f'<rect x="0" y="0" width="{width}" height="{height}" fill="white"/>')
    lines.append(
        f'<text x="{width/2:.1f}" y="35" text-anchor="middle" font-size="24" font-family="sans-serif">{html.escape(title)}</text>'
    )

    # axes
    y_zero = y_of(0.0)
    lines.append(
        f'<line x1="{margin_left}" y1="{margin_top}" x2="{margin_left}" y2="{margin_top + plot_h}" stroke="black" stroke-width="1"/>'
    )
    lines.append(
        f'<line x1="{margin_left}" y1="{y_zero:.2f}" x2="{margin_left + plot_w}" y2="{y_zero:.2f}" stroke="#777" stroke-width="1"/>'
    )

    # y ticks
    for i in range(6):
        v = min_v + (max_v - min_v) * i / 5.0
        y = y_of(v)
        lines.append(
            f'<line x1="{margin_left-5}" y1="{y:.2f}" x2="{margin_left}" y2="{y:.2f}" stroke="black"/>'
        )
        lines.append(
            f'<text x="{margin_left-10}" y="{y+4:.2f}" text-anchor="end" font-size="12" font-family="monospace">{v:.1f}</text>'
        )

    # bars
    for i, label in enumerate(labels):
        group_start = margin_left + i * group_w
        for j, (_, vals, color) in enumerate(series):
            v = vals[i]
            x = group_start + j * bar_w + bar_w * 0.2
            y = min(y_of(v), y_zero)
            h = abs(y_zero - y_of(v))
            lines.append(
                f'<rect x="{x:.2f}" y="{y:.2f}" width="{bar_w*0.6:.2f}" height="{max(h,1):.2f}" fill="{color}"/>'
            )

        label_x = group_start + group_w / 2
        lines.append(
            f'<text x="{label_x:.2f}" y="{margin_top + plot_h + 24}" text-anchor="end" transform="rotate(-30 {label_x:.2f} {margin_top + plot_h + 24})" font-size="12" font-family="sans-serif">{html.escape(label)}</text>'
        )

    # ylabel
    lines.append(
        f'<text x="20" y="{margin_top + plot_h/2:.2f}" transform="rotate(-90 20 {margin_top + plot_h/2:.2f})" text-anchor="middle" font-size="13" font-family="sans-serif">{html.escape(ylabel)}</text>'
    )

    # legend
    legend_x = margin_left
    legend_y = height - 60
    for name, _, color in series:
        lines.append(f'<rect x="{legend_x}" y="{legend_y}" width="18" height="18" fill="{color}"/>')
        lines.append(
            f'<text x="{legend_x+24}" y="{legend_y+14}" font-size="13" font-family="sans-serif">{html.escape(name)}</text>'
        )
        legend_x += 220

    lines.append('</svg>')
    output_path.write_text("\n".join(lines), encoding="utf-8")


def main():
    if len(sys.argv) < 3:
        print("usage: plot_idcp_bench.py <bench_csv> <output_dir>", file=sys.stderr)
        sys.exit(2)

    csv_path = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(parents=True, exist_ok=True)

    rows = read_rows(csv_path)
    labels = [row["scenario"] for row in rows]

    mem = [to_float(row, "mem_percent") for row in rows]
    flow = [to_float(row, "flow_percent") for row in rows]
    copy = [to_float(row, "copy_percent") for row in rows]
    runtime_lat = [to_float(row, "runtime_latency_percent") for row in rows]
    runtime_tput = [to_float(row, "runtime_throughput_percent") for row in rows]
    score = [to_float(row, "score_multiplier") for row in rows]
    flow_default = [to_float(row, "flow_default_mean_ns") for row in rows]
    flow_idcp = [to_float(row, "flow_idcp_mean_ns") for row in rows]

    svg_bar_chart(
        labels,
        [
            ("memory %", mem, "#2E86AB"),
            ("flow %", flow, "#A23B72"),
            ("copy %", copy, "#F18F01"),
        ],
        "IDCP vs Conventional: subsystem improvement (%)",
        "improvement % (higher is better)",
        output_dir / "subsystem_improvements.svg",
    )

    svg_bar_chart(
        labels,
        [
            ("runtime latency %", runtime_lat, "#1B998B"),
            ("runtime throughput %", runtime_tput, "#6A4C93"),
        ],
        "IDCP vs Conventional: runtime improvement (%)",
        "improvement % (higher is better)",
        output_dir / "runtime_improvements.svg",
    )

    svg_bar_chart(
        labels,
        [("score multiplier", score, "#0B4F6C")],
        "IDCP aggregate score multiplier (x)",
        "score_x",
        output_dir / "score_multiplier.svg",
    )

    svg_bar_chart(
        labels,
        [
            ("default flow ns", flow_default, "#7D8597"),
            ("idcp flow ns", flow_idcp, "#EF476F"),
        ],
        "Measured flow latency means (ns)",
        "mean flow latency (ns)",
        output_dir / "flow_means_ns.svg",
    )

    print(f"wrote graphs to {output_dir}")


if __name__ == "__main__":
    main()
