# cuda-ephemeral

Ephemeral task management for the Cocapn fleet.

Most work in the fleet is short-lived: answer a question, analyze a chunk of data, generate a report, probe a service, then vanish. These aren't vessels—they're *ephemeral tasks*. They spawn, do one thing, return a result, and disappear.

This crate provides the lifecycle management, priority scheduling, energy budgeting, and trust-gated access that ephemeral tasks need.

## Use Cases

- **One-shot queries** — A vessel receives a question from a human or another agent, spawns an ephemeral task to handle it, returns the answer, and the task is garbage-collected.
- **Fleet monitoring** — Short-lived monitoring probes that run for a few cycles, collect metrics, report back, and expire.
- **Test harnesses** — Automated test tasks spawned during fleet boot or on-demand, with strict TTL to prevent zombie tests.
- **A2A request handling** — When another agent sends a request, the fleet can dispatch it as an ephemeral task rather than assigning it to a persistent vessel.

## Related Crates

- `cuda-vm-scheduler` — Persistent vessel scheduling and resource allocation
- `cuda-atp-market` — Energy trading and task marketplace
- `cuda-trust` — Trust scoring and reputation for fleet participants

## Build / Test

```bash
cargo build
cargo test
```

## The Deeper Connection

Ephemeral computing is what comes after SaaS. Instead of long-running services that accumulate state and technical debt, most work should be *disposable*. Spawn a task, give it a budget and a deadline, get a result, let it die. The Cocapn fleet is built on this principle: persistent vessels provide stability and identity, but the actual *work* flows through ephemeral tasks that can't outlive their usefulness. This crate is the engine for that flow.
