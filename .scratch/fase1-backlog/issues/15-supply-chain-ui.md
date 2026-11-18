# 15 — Supply chain da árvore de UI (sem `cargo vet`)

**What to build:** Rodar `cargo deny --workspace check` e `cargo audit` sobre eframe/rfd; ajustar `deny.toml` com exceções individuais e justificadas (licenças de fontes embutidas, por exemplo), sem ignore genérico; avaliar desligar features padrão do eframe não usadas (`datepicker` de `egui_extras` não tem uso no código). `cargo vet` é dispensado (projeto solo) e registrado em ADR.

**Blocked by:** F1-13.

**Status:** PARCIAL (ADR-0021, 2026-10-11) — `cargo deny --workspace check` executado sobre a árvore real: `advisories ok, bans ok, licenses ok, sources ok` após ajustar `deny.toml` (3 licenças permissivas, fontes do egui só para `epaint_default_fonts`, 3 avisos com exceção justificada) e atualizar `yoke-derive` (retirada). **PENDING:** `cargo audit` (cobertura equivalente via banco RustSec já usada pelo deny), `cargo deny` com o toolchain pinado e a decisão de remover features não usadas do eframe.

**Status vigente (2026-11-04, espelho de `docs/rtm.md`; Gap origem: INC-03):** PARCIAL — detalhe na linha F1-15 da RTM, que é a autoridade. `cargo audit` e `cargo deny --workspace check` foram executados em 2026-10-29 (`docs/SUPPLY_CHAIN.md`); `cargo vet` está dispensado (ADR-0018 D-2).

**Origem:** AUD-13

- [ ] `cargo deny` e `cargo audit` verdes ou com exceções individuais justificadas em `deny.toml`.
- [ ] `docs/SUPPLY_CHAIN.md` e `LICENSES_THIRD_PARTY.md` refletem a árvore real.
- [ ] Features não usadas removidas apenas se `cargo check`/`cargo test` continuarem verdes.
