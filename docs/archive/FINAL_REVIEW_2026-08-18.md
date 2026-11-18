<!-- doc-type: review | status: historical -->
> **Status: HISTÓRICO / SUPERSEDED.** Este documento registra uma revisão anterior e não é fonte normativa do runtime atual. Consulte `docs/PROJECT_SPEC_v6_4_5.md` e `docs/FASE0_STATUS.md`.

# Final review — 2026-08-18 — supply-chain gates

## Findings from real user execution

1. `cargo check --locked --all-targets --all-features` passed.
2. `cargo deny 0.20.x` rejected the deprecated `licenses.unlicensed` key. Fixed by migrating to the current allow-list semantics.
3. `cargo audit` found RUSTSEC-2026-0233/0234/0235 in `rkyv 0.8.16`; the lock baseline is updated to `rkyv 0.8.17` and `rkyv_derive 0.8.17`.
4. `encoding 0.2.33` is unmaintained but is a mandatory direct dependency of `evtx 0.12.2`; it remains an explicit, reviewable exception rather than a security override.
5. `cargo vet` failed before the store was initialized; the documented path is `cargo vet init` / `just vet-init`, followed by human review of `supply-chain/`.

## Release interpretation

No vulnerability advisory is waived. The only advisory exception is the unmaintained `encoding` crate required by the selected EVTX parser. This exception must be reconsidered when `evtx` changes.
