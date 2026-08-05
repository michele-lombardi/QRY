#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 3 ]]; then
  printf 'Usage: %s PID [SAMPLES] [INTERVAL_SECONDS]\n' "$0" >&2
  exit 2
fi

process_id="$1"
sample_count="${2:-30}"
sample_interval="${3:-1}"

if [[ ! "$process_id" =~ ^[0-9]+$ || ! "$sample_count" =~ ^[1-9][0-9]*$ ]]; then
  printf 'PID and sample count must be positive integers.\n' >&2
  exit 1
fi
if [[ ! "$sample_interval" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  printf 'Interval must be a non-negative number of seconds.\n' >&2
  exit 1
fi
if ! kill -0 "$process_id" 2>/dev/null; then
  printf 'Process is not running: %s\n' "$process_id" >&2
  exit 1
fi

for ((sample = 1; sample <= sample_count; sample += 1)); do
  ps -p "$process_id" -o %cpu= -o rss=
  if ((sample < sample_count)); then
    sleep "$sample_interval"
  fi
done | awk '
  {
    cpu_sum += $1;
    rss_sum += $2;
    if ($1 > cpu_max) cpu_max = $1;
    if ($2 > rss_max) rss_max = $2;
    count += 1;
  }
  END {
    if (count == 0) exit 1;
    printf "samples=%d average_cpu_percent=%.2f max_cpu_percent=%.2f average_rss_mib=%.2f max_rss_mib=%.2f\n",
      count, cpu_sum / count, cpu_max, rss_sum / count / 1024, rss_max / 1024;
  }
'
