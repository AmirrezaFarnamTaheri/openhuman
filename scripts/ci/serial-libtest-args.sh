#!/usr/bin/env bash
# Preserve arbitrary libtest passthrough options without accepting a second
# thread-count option in CI's intentionally serialized suites.
#
# Input: EXTRA_ARGS (array parsed by test-rust-e2e.sh)
# Output: SERIAL_LIBTEST_ARGS (array for the serial test branch).
filter_serial_libtest_args() {
  SERIAL_LIBTEST_ARGS=()
  local arg
  local skip_value=0

  for arg in "${EXTRA_ARGS[@]}"; do
    if [ "$skip_value" -eq 1 ]; then
      # A missing value followed by another flag must fail, not be eaten.
      if [[ ! "$arg" =~ ^0*[1-9][0-9]*$ ]]; then
        echo "[rust-e2e] ERROR: --test-threads requires a positive integer value" >&2
        return 2
      fi
      skip_value=0
      continue
    fi
    case "$arg" in
      --test-threads)
        skip_value=1
        ;;
      --test-threads=*)
        # Invalid values are user errors. Do not suppress them while forcing 1.
        local value="${arg#--test-threads=}"
        if [[ ! "$value" =~ ^0*[1-9][0-9]*$ ]]; then
          echo "[rust-e2e] ERROR: --test-threads requires a positive integer value" >&2
          return 2
        fi
        ;;
      *)
        SERIAL_LIBTEST_ARGS+=("$arg")
        ;;
    esac
  done

  if [ "$skip_value" -eq 1 ]; then
    echo "[rust-e2e] ERROR: --test-threads requires a positive integer value" >&2
    return 2
  fi
}
