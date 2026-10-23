## Summary

- Add atomic edit and lane-move operations for pending `RunQueue` items.
- Add durable addressed-input admission through an optional `RunInboxStore` port.
- Validate persisted recipient lineage and commit input, deduplication identity,
  event sequence and queued receipt in one checked storage batch.
- Preserve current storage-port and import APIs. Declare the new collection at
  its initial version, and gate a Unix-only SDK test import on Unix.

## API or behavior changes

`edit_first_where` and `move_first_where` return false when a queued item has
already been consumed. Other pending items keep their order. Moving an item to
the steering lane requests its next safe delivery boundary.

`RunInboxStore::enqueue_input` addresses an exact run under a root conversation.
Retries with identical submission content return the original receipt; reuse
with different content fails. `pending_inputs` preserves admission order.

The receipt means **durably queued**. Recipient consumption, delivery
acknowledgement, worker continuation and shared budgets are follow-on work.

## Tests

Verified on upstream main `8dd8f3ffd584cfd8bb2ecbce8526e85e75dad5b2` plus this branch:

- `cargo fmt --all -- --check` — passed.
- `cargo test -p tinyagents-session --lib run_ledger::drivers:: -- --test-threads=1`
  — 72 passed, including duplicate admission, handle recreation, concurrent
  admission, order, root isolation and invalid lineage.
- `cargo test -p tinyagents-harness --lib run_queue:: -- --test-threads=1`
  — 26 passed.
- `cargo clippy -p tinyagents-session -p tinyagents-harness --all-targets -- -D warnings`
  — passed.

## Documentation

Updated the queue README and command-center README; the new public port and
types document their admission and delivery boundaries.
