# Relatório de alterações — 2026-11-14

**Pedido do operador:** ler todo o repositório, analisar o "Registro de problemas – execução local do v6.5.1" e planejar/desenvolver a correção, passo a passo.

**Ambiente de execução:** Linux x86_64 (Debian 13), `rustc 1.99.1` (58cf8f9c3, 2026-10-13), `cargo 1.99.1`, Python 3.13.5, cargo-audit 0.22.2, cargo-deny 0.20.2. Não há repositório Git neste diretório; o "commit-tree" das evidências é a árvore de trabalho de 2026-11-14.

## Resultado por item do registro

| ID | Antes | Depois | Evidência |
|---|---|---|---|
| BUILD-01 | `just e2e`/`just demo` falhavam no build; e2e nunca rodava | `-p dfir-app -p dfir-engine`; `just e2e` executado com engine e EVTX reais | `EVD-002` (110/110) |
| GUI-01 | seletor bloqueava a UI e falhava em silêncio | seletor fora da thread, caminho digitado e arrastar-e-soltar, mensagens; docs Linux | `EVD-001` (testes), `docs/OPERATION.md` §3D |
| SEC-01 | `evtx 0.12.3` / `RUSTSEC-2026-0431`; `encoding` no grafo | `evtx 0.15.1`; advisory resolvido; `encoding` removido | `EVD-002`, `EVD-003` |
| SEC-02 | `libsqlite3-sys 0.37.0` / `RUSTSEC-2026-0459` | sem correção possível; exceção temporária justificada, revisão até 2026-12-31 | `EVD-003` |
| TOOL-01 | `supply-chain` parava no `cargo audit`; `cargo deny` não rodava | as duas etapas rodam e passam; `async-executor` yanked corrigido | `EVD-003` |
| GFX-01 | erros EGL sem dizer o adaptador escolhido | adaptador registrado no log; erros mantidos como diagnóstico | compilação + `docs/NATIVE_UI_ARCHITECTURE.md` |
| DOC-01 | `rust-version`/docs em 1.97; issues com o pin antigo | `rust-version = "1.99"` e docs atualizados; notas nos tickets | `EVD-001`, `just doc-gate` |

## Correções de código

- `engine/Cargo.toml`: `evtx 0.15.1` (`default-features = false`), com comentário apontando os dois RUSTSEC.
- `Cargo.lock`: `evtx 0.12.3 → 0.15.1`, `encoding*`/`skeptic`/`pulldown-cmark`/`glob`/`cargo_metadata`/`error-chain`/`bytecount` removidos, `encoding_rs`/`multiversion_no_op` adicionados, `async-executor 1.14.1`, `utf16-simd 0.2.0`.
- `src/gui/mod.rs`: corrigido o uso de `DroppedFile::path()` (erro de compilação introduzido pela correção de GUI-01) e mantido o registro do adaptador `wgpu`.
- `src/gui/evidence.rs`: o aviso de múltiplos arquivos soltos passou a ser gravado **depois** de `begin_import` (que limpa avisos), então o operador realmente o vê; três testes novos cobrem caminho vazio/pasta, múltiplos arquivos e importação concorrente.
- `src/gui/file_dialog.rs`: já continha o seletor fora da thread; permaneceu sem alteração funcional nesta rodada.
- `scripts/e2e_headless.py`: não estoura mais `IndexError` quando o EVTX não gera candidatos; filtros usam o Event ID mais frequente do arquivo; o EVTX truncado passou a ser menor que o arquivo (`max(70_000, 55%)` devolvia o arquivo inteiro para arquivos < 70 KB); a conferência do nome no relatório entende o escape Markdown (`\_`).
- `scripts/validate_fase0.py`: exige `evtx >= 0.15.1` (em vez da string fixa 0.12.2), com a razão do advisory.
- `deny.toml` e `.cargo/audit.toml`: uma exceção de vulnerabilidade (`RUSTSEC-2026-0459`) com análise de alcance; exceção de `encoding` removida.
- `.gitignore`: amostras `.evtx` na raiz não entram no pacote.

## Documentação

- `README.md`, `CHANGELOG.md`, `PROMPT_NOVA_CONVERSA.md`, `docs/FASE1_STATUS.md`, `docs/rtm.md`, `docs/RELEASE_GATE.md`, `docs/EVIDENCE_REGISTER.md`.
- `docs/SUPPLY_CHAIN.md`: estado vigente, `evtx 0.15.1`, exceção de `libsqlite3-sys`, rodada de 2026-11-14.
- `docs/OPERATION.md` §3D: dependências Linux do diálogo nativo e caminhos alternativos.
- `docs/NATIVE_UI_ARCHITECTURE.md`: módulos `file_dialog.rs`, `deliverables.rs`, `gaps.rs`, `plan.rs`; seletor fora da thread; log do adaptador.
- `docs/DEMO.md`: o e2e aceita qualquer EVTX, mas as etapas de detecção dependem do conteúdo; amostra `CA_hashdump…` não deve ser distribuída.
- `docs/TOOLCHAINS.md`: Rust 1.99.1 e `evtx 0.15.1`.
- `LICENSES_THIRD_PARTY.md`: inventário regerado do lock (659 pacotes, 656 de terceiros), sem `encoding`, com `encoding_rs`.
- `Registro de problemas – execução local do v6.5.1.md`: tabela de resolução e nota por item.
- `.scratch/fase1-backlog/issues/25,26,30,32,34,35`: nota de 2026-11-14 e itens de verificação fechados onde há evidência.
- `docs/evidence/2026-11-14-*.txt`: resumos das quatro execuções.

## Comandos executados (todos nesta árvore)

```bash
just verify
just e2e /caminho/security.evtx
just supply-chain
just doc-gate
cargo metadata --locked --format-version 1
cargo tree --locked -e features
cargo audit
cargo audit --no-yanked
cargo deny --workspace check
cargo update -p libsqlite3-sys --precise 0.39.0   # falha esperada, documenta a impossibilidade
python3 scripts/e2e_headless.py --evtx CA_hashdump_4663_4656_lsass_access.evtx
```

Resultados: `just verify` PASS (app 294 + 1 ignorado, engine 83 + 7, protocolo 26; nextest 410/410 com 1 skipped; clippy limpo); e2e 110/110; `cargo audit` 0 vulnerabilidades e 5 avisos informativos; `cargo deny --workspace check` ok; `doc-gate` PASS. O e2e com a amostra de 2 eventos terminou com 100/103 PASS e 3 FAIL honestos, todos dependentes do conteúdo (sem candidatos de detecção), sem crash.

## Riscos e pendências

- **SEC-02 não está corrigido:** `RUSTSEC-2026-0459` é uma vulnerabilidade real no SQLite embutido, aceita temporariamente. Reavaliar até 2026-12-31, quando houver `sqlx` que admita `libsqlite3-sys >= 0.39.0`.
- **Não executado:** Windows (Job Object), Linux aarch64, janela real da GUI, incidente real; fixtures EVTX sanitizadas (F1-16); testes de `RF-003`/`RF-027`.
- Nada foi declarado `ACCEPTED`; os gates executados são `PASS` apenas para Linux x86_64 e esta árvore.

## Rodada da tarde — upgrade `eframe`/`egui` 0.37.0 + `cargo update` geral (item 1 do plano do operador)

**Pedido do operador:** atualizar todas as dependências, começando pelo `eframe` mais recente, de preferência eliminando os três avisos informativos `zbus`/`zvariant` (RUSTSEC-2026-0370/0371/0372).

- `eframe`/`egui`/`egui_extras`/`egui_kittest` 0.36.2 → **0.37.0**; `rfd` 0.17.2 → **0.18.0**. `eframe` passou a `default-features = false` com `default_fonts`, `wgpu`, `x11`, `wayland`, `clipboard`, `links`: em 0.37 a feature `accesskit` do `egui-winit` é opcional e, desligada, a cadeia D-Bus/AT-SPI (`zbus`, `zvariant`, `zbus_names`, `zvariant_derive`, `atspi`, `accesskit_unix`, `ashpd`, `sctk-adwaita`, `ttf-parser`) **sai do grafo de build** (lock: 615 → 612 pacotes). A pendência de política das três exceções informativas de UI foi assim **resolvida pela remoção**, não por waiver.
- `cargo update --workspace`: 0 mudanças pendentes (já estávamos nas máximas compatíveis com rustc 1.99); minor updates aplicados: `jiff 0.2.38→0.2.44`, `toml 1.1.8→1.2.0`, `uuid 1.29.0→1.29.2` (`defmt` saiu do grafo). Os 52 pacotes restantes disponíveis exigem `rust-version > 1.99`.
- `deny.toml`: removidas as 4 exceções mortas (RUSTSEC-2026-0370/0371/0372 e RUSTSEC-2026-0192); restam `RUSTSEC-2026-0459` (SEC-02) e `RUSTSEC-2025-0141` (bincode via yara-x).
- Código: o app compilou sem mudança no binário; os testes de GUI (`egui_kittest 0.37`) exigiram harness com `allow_missing_glyphs()` (padrão `MissingGlyphPolicy::Panic` quebra no glifo `▸`) e nome acessível em cada campo (`labelled_by` nos `TextEdit`, `ComboBox::from_label`), por causa da checagem nova de acessibilidade. Ajustes em `src/gui/mod.rs` (novo `test_harness::builder`, helpers `field_text*`, `enum_combo` com `from_label`) e nos módulos de view.
- **Resultados (EVD-005, EVD-006, EVD-007):** `just verify` PASS (nextest 410/410, clippy `-D warnings` limpo, fmt ok); `just e2e <security.evtx>` **110/110**; `cargo audit` 0 vulnerabilidades (1 aviso permitido: bincode); `cargo deny --workspace check` `advisories/bans/licenses/sources ok`.
- **Não repetido nesta rodada:** Windows, Linux aarch64, janela real da GUI.

## Rodada da noite — itens p2–p6 do plano do operador (decisões pendentes resolvidas em 2026-11-14T22:00Z)

- **p2 — waivers formais** em `docs/EVIDENCE_REGISTER.md` (formato F.14, `approved-by: operador do projeto, sessão 2026-11-14T22:00Z`, `valid-until: 2026-12-31`): `WVR-001` Windows (Job Object só compilado; condição de saída = `engine-smoke` + e2e em Windows com `SANDBOX_NOT_FULLY_APPLIED`), `WVR-002` aarch64, `WVR-003` CI automático (F1-14).
- **p3 — `docs/DEFINITION_OF_READY.md`:** 9 itens `[x]` com `EVD-001`..`EVD-009` e `WVR-001`..`WVR-004` (EVI-01 satisfeito; `just doc-gate` valida as citações).
- **p4 — SEC-02 → `WVR-004`:** `RUSTSEC-2026-0459` (`libsqlite3-sys 0.37.0`) formalizado; ponteiros em `deny.toml` e `.cargo/audit.toml`; revisão até 2026-12-31.
- **p5 — verificação real de hardware/GUI (`EVD-009`):** `timeout 25 ./target/debug/dfir-app` no Xorg `:0` real do host — janela abriu e ficou aberta (exit 124); log registrou `native UI GPU adapter`: Mesa Intel HD Graphics 2000 (SNB GT1), backend Gl, IntegratedGpu, 2 candidatos (llvmpipe era o outro); erros EGL 0x3009/0x3005 do wgpu_hal continuaram visíveis e interpretáveis pela linha do adaptador (GFX-01). Aviso cosmético: glifo `📁` ausente das fontes padrão.
- **p6 — políticas operacionais** em `docs/OPERATION.md`: §3.1.1 (retenção de `backups/`: app nunca apaga; manter as duas últimas; limpeza manual após `backup-verify`) e §5 (pacote não distribui `.evtx`; `security.evtx` fornecido pelo operador com SHA-256 `5f29a03c…9a4d8d`; F1-16 de fixtures sanitizadas continua PLANNED).
- **p7 — documentação e gates finais (2026-11-15T00:29Z):** `WVR-004` referenciado nos comentários de `deny.toml` (linha 8) e `.cargo/audit.toml`; `just doc-gate` PASS (Fase 0 0 erros/0 avisos + doc_gate PASS, valida as citações `EVD-*`/`WVR-*`); `cargo fmt --all -- --check` limpo; `cargo audit` 0 vulnerabilidades (só o aviso permitido `RUSTSEC-2025-0141`/bincode); `cargo deny --workspace check` `advisories ok, bans ok, licenses ok, sources ok` (612 pacotes no lock). `docs/rtm.md`, `CHANGELOG.md`, `docs/FASE1_STATUS.md` e `docs/RELEASE_GATE.md` atualizados com a rodada p2–p6.
