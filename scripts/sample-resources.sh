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

is_macos=0
if [[ "$(uname -s)" == Darwin ]]; then
  is_macos=1
fi

for ((sample = 1; sample <= sample_count; sample += 1)); do
  resource_sample="$(ps -p "$process_id" -o %cpu= -o rss=)"
  physical_footprint=0
  if ((is_macos)); then
    # RSS excludes compressed allocations: it can look small during a large
    # native leak. Match Activity Monitor's physical footprint as well.
    physical_footprint="$(footprint "$process_id" -f bytes | awk '
      $1 == "phys_footprint:" { print $2; found = 1; exit }
      END { if (!found) exit 1 }
    ')"
  fi
  printf '%s %s\n' "$resource_sample" "$physical_footprint"
  if ((sample < sample_count)); then
    sleep "$sample_interval"
  fi
done | awk '
  {
    cpu_sum += $1;
    rss_sum += $2;
    if ($1 > cpu_max) cpu_max = $1;
    if ($2 > rss_max) rss_max = $2;
    footprint_sum += $3;
    if ($3 > footprint_max) footprint_max = $3;
    if (count == 0) footprint_first = $3;
    footprint_last = $3;
    count += 1;
  }
  END {
    if (count == 0) exit 1;
    printf "samples=%d average_cpu_percent=%.2f max_cpu_percent=%.2f average_rss_mib=%.2f max_rss_mib=%.2f",
      count, cpu_sum / count, cpu_max, rss_sum / count / 1024, rss_max / 1024;
    if (footprint_max > 0) {
      printf " average_footprint_mib=%.2f max_footprint_mib=%.2f footprint_delta_mib=%.2f",
        footprint_sum / count / 1048576, footprint_max / 1048576,
        (footprint_last - footprint_first) / 1048576;
    }
    printf "\n";
  }
'
