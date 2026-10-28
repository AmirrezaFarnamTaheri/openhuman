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
      skip_value=0
      continue
    fi
    case "$arg" in
      --test-threads)
        skip_value=1
        ;;
      --test-threads=*)
        ;;
      *)
        SERIAL_LIBTEST_ARGS+=("$arg")
        ;;
    esac
  done

  if [ "$skip_value" -eq 1 ]; then
    echo "[rust-e2e] ERROR: --test-threads requires a value" >&2
    return 2
  fi
}
