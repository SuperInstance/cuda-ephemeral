# MAINTENANCE.md

## Architecture

`cuda-ephemeral` is built around three core concepts:

1. **EphemeralTask** — The task itself: an ID, a purpose, a payload, energy budget, TTL, priority, and trust requirement.
2. **TaskRegistry** — The central store. Holds all tasks, enforces concurrency and energy limits, handles state transitions (spawn → complete/cancel/expire).
3. **TaskQueue** — A simple priority queue (sorted Vec, no heap dependency) for scheduler-facing task ordering.

`lifecycle.rs` provides the scheduling logic: `next_task` picks the best task a vessel can handle, `check_expiry` handles TTL-based expiration, and `energy_reclaim` recovers unused energy from dead tasks.

## Why Ephemeral Tasks Matter

Most fleet work is short-lived. A persistent vessel might live for thousands of cycles, but the individual tasks it handles—answering questions, analyzing data, probing services—are measured in single or double-digit cycle counts. Modeling these as ephemeral tasks rather than long-lived processes gives us:

- **Automatic cleanup** via TTL-based expiry
- **Resource bounds** via energy budgets and concurrency limits
- **Trust-gated access** so sensitive tasks only go to trusted vessels
- **Priority scheduling** so important work gets done first

## Design Decisions

- **TTL-based expiry**: Tasks auto-expire after their TTL in cycles. No explicit cleanup needed. `check_expiry` is called each cycle by the scheduler.
- **Priority queue**: Simple sorted Vec rather than a binary heap. The number of concurrent ephemeral tasks is small (bounded by `max_concurrent`), so O(n) insertion is fine. Keeps dependencies minimal.
- **Trust gating**: Each task has a `trust_required` threshold. A vessel's trust score must meet or exceed this to pick up the task. Prevents low-trust or unknown vessels from handling sensitive work.
- **Energy budgeting**: Ephemeral tasks draw from a shared energy pool. When the pool is depleted, no new tasks spawn. Expired/failed tasks release their unused energy back.
- **No unwrap in logic**: All fallible operations return `Result`. Only tests use `unwrap`.

## Future Directions

- **Cascading tasks**: A completed task can spawn child tasks, forming a task tree. The `parent_id` field is already in place.
- **Task trees**: Build full DAGs of dependent ephemeral tasks with propagation of failure and cancellation.
- **Result caching**: Cache completed task results keyed by (purpose, payload_hash) to avoid redundant work.
- **Energy market integration**: Allow ephemeral tasks to bid for energy on the ATP market when the pool is low.
- **Distributed registry**: Spread the task registry across multiple fleet nodes with consensus for high-availability fleets.
