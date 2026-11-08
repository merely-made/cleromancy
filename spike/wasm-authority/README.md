# Divination authority as a wasm component: the spike

Throwaway, for Mere's cycle breaks P3 step 2. Not the guest; the real `domain-authority` world, review
binding and host belong to P3 step 3 after Mark's review. Findings are in Mere's
`design_docs/mere_docs/implementation_strategy/2026-11-06_cycle_breaks_plan.md` (Findings, "P3 spike").

Reproduce (toolchain 1.97.1, from this directory):

```sh
# 1. The corpus: the seven tests' graphs plus single-leaf tampers, with native verdicts.
(cd ../../core && CLEROMANCY_AUTHORITY_CORPUS=/tmp/corpus CLEROMANCY_AUTHORITY_TAMPERS=150 \
  cargo +1.97.1 test --locked --test authority_corpus)
# 2. The component (a wasm32-wasip2 cdylib is a component) and the harness.
cargo +1.97.1 build --release --target wasm32-wasip2 -p authority-guest
cargo +1.97.1 build --release -p authority-host
# 3. Run it: verdicts, per-call time, fuel, memory.
target/release/authority-host target/wasm32-wasip2/release/authority_guest.wasm /tmp/corpus
# A fuel cut-off: AUTHORITY_FUEL=20000000 traps, and the instance cannot be re-entered.
```
