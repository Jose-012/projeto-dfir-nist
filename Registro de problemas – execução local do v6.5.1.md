# Registro de problemas – execução local do v6.5.1

2026-11-14 · projeto-dfir-nist-v6\_5\_1-2026-11-09 · **atualizado em 2026-11-14: correções aplicadas e verificadas; ver "Resolução" em cada item**

**Ambiente:** Debian 13 (Mesa 25.0.7-2+deb13u1), X11, Intel HD 2000 (SNB GT1), rustc 1.99.1, `dfir-app` 0.1.0. `just verify` passou sem erros.

**Legenda:** *verificado* = confirmado lendo o código/lockfile do pacote. *hipótese* = precisa ser confirmado pelo agente antes de mexer.

## Resumo

| ID | Área | Gravidade | Efeito |
| --- | --- | --- | --- |
| BUILD-01 | justfile | Alta | `just e2e` e `just demo` não compilam; e2e nunca roda |
| GUI-01 | GUI / Linux | Alta | Botão "Importar evidência…" não faz nada e pode travar a janela |
| SEC-01 | Supply chain | Média (5.5) | RUSTSEC-2026-0431 em `evtx` 0.12.3 |
| SEC-02 | Supply chain | Média (6.8) | RUSTSEC-2026-0459 em `libsqlite3-sys` 0.37.0 |
| TOOL-01 | justfile | Média | `just supply-chain` para no `cargo audit`; `cargo deny` nunca roda |
| GFX-01 | GUI / gráficos | Baixa | Erros EGL no log; adaptador escolhido não é registrado |
| DOC-01 | Manifestos / docs | Baixa | `rust-version` e menções a 1.97.1 desatualizadas |

Ordem sugerida: BUILD-01 → GUI-01 → TOOL-01 → SEC-01 e SEC-02 → GFX-01 → DOC-01.

## Resolução (2026-11-14)

Execução em Linux x86_64, rustc/cargo 1.99.1, com `--locked`. Evidências em
`docs/EVIDENCE_REGISTER.md` (`EVD-001` a `EVD-003`) e detalhe em
`docs/SUPPLY_CHAIN.md`, `docs/OPERATION.md` §3D, `docs/NATIVE_UI_ARCHITECTURE.md` e
`CHANGELOG.md`.

| ID | Estado | Correção e evidência |
|---|---|---|
| BUILD-01 | **Corrigido e verificado** | `justfile` e `docs/DEMO.md` usam `cargo build -p dfir-app -p dfir-engine`; `just e2e` executado com engine e EVTX reais, **110/110** (`EVD-002`). O script ficou robusto para qualquer EVTX: não estoura mais sem candidatos, usa os Event IDs presentes e trunca o EVTX de fato. |
| GUI-01 | **Corrigido e verificado por testes** | `src/gui/file_dialog.rs` roda o `rfd` fora da thread da UI; botão nunca fica mudo, e há caminho digitado + arrastar-e-soltar com a mesma validação. Testes de `egui_kittest`/unidade cobrem os caminhos alternativos (`EVD-001`). Dependências Linux documentadas em `docs/OPERATION.md` §3D. |
| SEC-01 | **Corrigido e verificado** | `evtx 0.12.3 → 0.15.1`; `RUSTSEC-2026-0431` resolvido e `encoding`/`RUSTSEC-2021-0153` fora do grafo. e2e com 2 261 eventos e 6 candidatos, igual à versão anterior (`EVD-002`). |
| SEC-02 | **Risco aceito e documentado (não corrigido)** | Sem `sqlx` publicado que admita `libsqlite3-sys >= 0.39.0`; exceção temporária em `deny.toml` e `.cargo/audit.toml`, com alcance e revisão até 2026-12-31. **Não é prova de inexploração.** |
| TOOL-01 | **Corrigido e verificado** | `just supply-chain` executa `cargo audit` e `cargo deny --workspace check`; ambos passam com a exceção documentada de SEC-02 (`EVD-003`). `async-executor 1.14.0` (yanked) → 1.14.1. |
| GFX-01 | **Corrigido (diagnóstico) e documentado** | O adaptador `wgpu` escolhido é registrado na inicialização (`adapter`, `backend`, `device_type`, `driver`). Os erros de `wgpu_hal` continuam visíveis, por decisão: são diagnóstico real e a linha do adaptador permite interpretá-los. |
| DOC-01 | **Corrigido** | `rust-version = "1.99"` nos três manifests; `README.md`, `PROMPT_NOVA_CONVERSA.md`, `docs/TOOLCHAINS.md`, `docs/SUPPLY_CHAIN.md` e `LICENSES_THIRD_PARTY.md` atualizados. Itens abertos dos tickets 34 e 35 marcados; menções a 1.97.1 em ADRs/logs históricos preservadas como registro. |

**O que continua aberto:** Windows, Linux aarch64, incidente real, fixtures EVTX versionadas (F1-16), testes de `RF-003`/`RF-027`, 3 exceções informativas de UI no `deny.toml` (decisão do operador) e a exceção de SEC-02 (reavaliar até 2026-12-31). O checklist externo citado em DOC-01 não está neste repositório; o item equivalente do ticket 34 foi corrigido em `.scratch/fase1-backlog/issues/34-cli-headless.md`.

## BUILD-01 – `just e2e` e `just demo` falham no build

**Comando:** `just e2e CA_hashdump_4663_4656_lsass_access.evtx`

**Erro:** `error: no bin target named 'dfir-engine' in default-run packages` (cargo sugere o pacote `dfir-engine`), seguido de `recipe e2e failed ... exit code 101`.

**Causa (verificado):** `justfile:62` (`e2e`) e `justfile:67` (`demo`), além de `docs/DEMO.md:20`, usam `cargo build --locked --bin dfir-app --bin dfir-engine`. A raiz é pacote e workspace ao mesmo tempo; sem `-p` ou `--workspace`, o cargo só procura binários em `dfir-app`. O binário `dfir-engine` está no pacote `engine/`.

**Correção sugerida:** trocar por `cargo build --locked -p dfir-app -p dfir-engine` nos três lugares. Conferir que o default de engine em `scripts/e2e_headless.py` (o `--app` padrão é `target/debug/dfir-app`) aponta para `target/debug/dfir-engine`, ou que `DFIR_ENGINE_PATH` é passado.

**Aceite:** `just e2e <arquivo.evtx>` compila, executa o script e imprime PASS/FAIL por etapa. Registrar a execução em `docs/EVIDENCE_REGISTER.md`.

**Observação:** o `.evtx` usado está na raiz do pacote. Confirmar se é um arquivo real ou sanitizado antes de distribuir o pacote (liga com a decisão da fixture sanitizada, F1-16).

**Resolução (2026-11-14):** `just e2e` executado com o `security.evtx` de referência, **110/110** (`EVD-002`); o mesmo comando com o `CA_hashdump…evtx` (2 eventos) roda até o fim e imprime PASS/FAIL honestos. O script foi corrigido para não estourar sem candidatos, para usar os Event IDs presentes e para truncar de fato o EVTX. Sobre a observação: é uma **amostra pública real** (2 eventos 4663/4656, computador `MSEDGEWIN10`), não sanitizada; a procedência não foi estabelecida. Ela **não deve ser distribuída** no pacote (`.gitignore` cobre `/*.evtx`; nota em `docs/DEMO.md`). A fixture sanitizada continua F1-16.

## GUI-01 – Seletor de arquivo não abre; falha silenciosa

**Log (17:18:20):**

- `OpenFile failed: org.freedesktop.DBus.Error.NoReply`
- `Using zenity fallback`
- `Failed to pick file with zenity: No such file or directory (os error 2)`

**Efeito:** o clique em "📥 Importar evidência…" não importa nada e não mostra erro, porque `rfd` devolve `None` tanto para cancelamento quanto para falha.

**Causa:**

- *verificado:* o diálogo é chamado de forma síncrona dentro do frame da UI em `src/gui/evidence.rs:199-201`, `src/gui/export.rs:30` e `src/gui/deliverables.rs:141`. Isso bloqueia a thread da UI até o diálogo responder ou estourar o tempo.
- *hipótese:* o `xdg-desktop-portal` (ou seu backend) não está rodando ou não responde nesta sessão, e o `zenity` não está instalado. Medir quanto tempo a janela fica travada (o timeout padrão do D-Bus é \~25 s).

**Diagnóstico no host:** `which zenity`, `echo $XDG_CURRENT_DESKTOP`, `systemctl --user status xdg-desktop-portal`, `busctl --user list | grep -i portal`.

**Correção de ambiente (a validar):** `sudo apt install xdg-desktop-portal xdg-desktop-portal-gtk zenity`, depois reiniciar a sessão ou o serviço do portal.

**Correção de código sugerida:**

- Chamar o diálogo fora da thread da UI (por exemplo `rfd::AsyncFileDialog` ou thread dedicada, com resultado por canal) e manter o estado `importing`.
- Oferecer alternativa que não dependa de diálogo: campo para colar o caminho e/ou arrastar e soltar o arquivo na janela.
- Documentar as dependências de Linux em `docs/OPERATION.md`.

**Aceite:** sem portal e sem zenity, a janela não congela e a pessoa consegue importar por outro caminho; com os pacotes instalados, o diálogo abre e a importação funciona.

**Resolução (2026-11-14):** o diálogo roda em `src/gui/file_dialog.rs` (thread + canal) e é drenado por quadro; a UI nunca espera o D-Bus. Há campo de caminho digitado e arrastar-e-soltar, todos por `request_import` (caminho vazio/pasta/importação concorrente recusados com mensagem; múltiplos arquivos avisam). Dependências Linux e diagnóstico em `docs/OPERATION.md` §3D. Testes novos em `src/gui/evidence.rs` cobrem os caminhos alternativos; a validação em janela real sem portal/zenity fica como verificação do operador.

## SEC-01 – RUSTSEC-2026-0431 (`evtx` 0.12.3)

**Aviso:** alocação de memória sem limite ao processar arquivo `.wevtcache`. Severidade 5.5. Corrigido em `evtx >= 0.15.1`.

**Estado atual (verificado):** `engine/Cargo.toml` declara `evtx = { version = "0.12.2", default-features = false }`; o `Cargo.lock` resolve 0.12.3. O `deny.toml` e `docs/SUPPLY_CHAIN.md` afirmam que não há `evtx` mais novo; essa premissa agora está desatualizada.

**Passos sugeridos:**

- `cargo tree -i evtx` e `cargo tree -i encoding` para ver o grafo.
- Subir `evtx` para >= 0.15.1 no engine e adaptar a API, se mudou.
- Rodar `just verify`, `just engine-protocol-smoke` e o e2e com o EVTX real; comparar contagem de eventos e avisos com a versão anterior.
- Reavaliar `RUSTSEC-2021-0153` (`encoding`): pode sair do grafo com o `evtx` novo, e então a exceção no `deny.toml` deve ser removida.
- Atualizar `deny.toml`, `docs/SUPPLY_CHAIN.md` e `LICENSES_THIRD_PARTY.md`.

**Se a atualização não for possível agora:** exceção no `deny.toml` com análise de alcance (o engine chega a processar `.wevtcache`?) e os controles compensatórios (sandbox e limite de memória do engine, ADR-0024). *hipótese:* o engine só lê `.evtx`; confirmar no código.

**Resolução (2026-11-14):** atualizado para `evtx 0.15.1` (`default-features = false`); a API usada não mudou. `cargo audit` não reporta mais `RUSTSEC-2026-0431`; `encoding`/`RUSTSEC-2021-0153` saiu do grafo e a exceção foi removida do `deny.toml`. A hipótese foi confirmada: o engine só chama `EvtxParser::from_path`/`serialized_records` e não abre `.wevtcache`. e2e: 2 261 eventos, 6 candidatos, sem avisos (`EVD-002`).

## SEC-02 – RUSTSEC-2026-0459 (`libsqlite3-sys` 0.37.0)

**Aviso:** vem do SQLite em C (CVE-2026-119375). Severidade 6.8. Corrigido em `libsqlite3-sys >= 0.39.0`.

**Estado atual (verificado):** `Cargo.lock` resolve `libsqlite3-sys` 0.37.0, dependência de `sqlx-sqlite` 0.9.0 (raiz usa `sqlx 0.9.0`, features `sqlite`, `migrate`, `macros`, `chrono`).

**Passos sugeridos:**

- `cargo tree -i libsqlite3-sys`.
- `cargo update -p libsqlite3-sys` só funciona se o intervalo exigido pelo `sqlx-sqlite` admitir 0.39; senão procurar versão do `sqlx` que admita.
- Confirmar se o SQLite é embutido ou do sistema (*hipótese:* embutido) e qual versão de `sqlite_version()` o app usa em runtime.
- Se não houver versão corrigida disponível: exceção temporária no `deny.toml` com justificativa e data de reavaliação.
- Rodar `just verify` e os testes de migração.

**Resolução (2026-11-14):** `cargo tree -i libsqlite3-sys` confirma `sqlx-sqlite 0.9.0 → libsqlite3-sys 0.37.0`; o `sqlx-sqlite` exige `>=0.30.1,<0.38.0` e `sqlx 0.9.0` é a última versão, então `cargo update -p libsqlite3-sys --precise 0.39.0` falha. O SQLite é **embutido** (`bundled`). **Não há correção disponível agora:** exceção temporária em `deny.toml` e `.cargo/audit.toml`, com análise de alcance (banco local em WAL no volume protegido, operador único, sem banco/`-wal` externo) e revisão até 2026-12-31. Migrations e `just verify` rodaram (`EVD-001`). Isto é risco aceito e documentado, **não** `VERIFIED`.

## TOOL-01 – `just supply-chain` aborta antes do `cargo deny`

**Efeito:** `cargo audit --no-yanked` sai com código 1 (2 vulnerabilidades, 6 avisos permitidos) e a receita `supply-chain` encerra sem executar `just licenses` (`cargo deny --workspace check`). Ou seja, `cargo deny` ainda não foi rodado neste pacote.

**Passos sugeridos:**

- Rodar `just licenses` isoladamente e anexar a saída.
- Decidir se `supply-chain` deve rodar as duas etapas e falhar só no final, para o relatório sair completo.
- Os avisos *unmaintained* (`bincode`, `encoding`, `ttf-parser`, `zbus_names`, `zvariant`, `zvariant_derive`) já têm exceção no `deny.toml`. As três de `zbus`/`zvariant` seguem pendentes de decisão no checklist.

**Resolução (2026-11-14):** `just supply-chain` roda as duas etapas e termina em 0: `cargo audit` sem vulnerabilidades (5 avisos informativos; `encoding` saiu) e `cargo deny --workspace check` com `advisories/bans/licenses/sources ok`, usando a exceção documentada de SEC-02 (`EVD-003`). `async-executor 1.14.0` (yanked) foi para 1.14.1. A decisão sobre as três exceções de UI continua com o operador.

## GFX-01 – Erros EGL e escolha de adaptador

**Log (17:17:35):** dois `ERROR` de `wgpu_hal::gles::egl` em `eglCreateContext` (códigos `0x3009` e `0x3005`). Em seguida o `egui_wgpu` lista 2 adaptadores: Vulkan/CPU (`llvmpipe`) e GL/IntegratedGpu (`Mesa Intel HD Graphics 2000`). A janela abriu normalmente.

**Pontos:**

- *hipótese:* o erro vem do `wgpu` tentando criar contexto GLES nessa GPU antiga e depois caindo para outro backend. Não afeta a função, mas polui o log de produção com `ERROR`.
- O log não diz qual adaptador foi escolhido. Se for o `llvmpipe` (renderização por CPU), a UI pode ficar lenta.
- `WARN` de `XSETTINGS` é inofensivo; ignorar.

**Passos sugeridos:**

- Registrar o adaptador em uso (`wgpu_render_state` do eframe) no log de inicialização.
- Comparar `WGPU_BACKEND=gl` e `WGPU_BACKEND=vulkan` em fluidez antes de mudar qualquer padrão.
- Considerar renderer `glow` para hardware antigo, só depois de medir.
- Decidir se erros de `wgpu_hal` entram no arquivo de log ou são filtrados no `EnvFilter`.

**Resolução (2026-11-14):** `gui/mod.rs::log_renderer` registra o adaptador escolhido (`adapter`, `backend`, `device_type`, `driver`, número de candidatos). Decisão: os erros de `wgpu_hal` **continuam** no log, porque são diagnóstico real; a linha do adaptador permite saber se o backend caiu para CPU (`llvmpipe`). Não foi medido `WGPU_BACKEND=gl` vs `vulkan` nem trocado o renderer: isso depende de medição no hardware do operador. Documentado em `docs/NATIVE_UI_ARCHITECTURE.md`.

## DOC-01 – Versão do Rust e referências desatualizadas

- `rust-toolchain.toml` está em 1.99.1, mas `Cargo.toml` (raiz) e `engine/Cargo.toml` ainda têm `rust-version = "1.97"`. Conferir também `protocol/Cargo.toml`. O checklist registra a decisão de subir; falta aplicar nos manifestos e em `docs/TOOLCHAINS.md` / `toolchains.toml`.
- Várias issues em `.scratch/fase1-backlog/issues/` (25, 26, 30, 32, 34, 35, entre outras) citam "1.97.1 pinado". Atualizar para 1.99.1 e, onde couber, citar a execução de `just verify` como evidência (`EVD-*`).
- No checklist colado, o item do `just e2e` está marcado `[X]` mas diz "não pôde ser executado"; deveria estar aberto, com referência a BUILD-01.

**Resolução (2026-11-14):** `rust-version = "1.99"` nos três manifests (já estava; confirmado) e `docs/TOOLCHAINS.md` atualizado. `README.md`, `PROMPT_NOVA_CONVERSA.md`, `docs/SUPPLY_CHAIN.md` e `LICENSES_THIRD_PARTY.md` também foram atualizados. As menções a 1.97.1 em ADRs e logs históricos foram preservadas de propósito: registram o que foi executado na época. Nos tickets 25, 26, 30, 32, 34 e 35, uma nota de 2026-11-14 aponta o pin atual; o item do `just e2e` no ticket 34 foi marcado `[x]` para Linux x86_64 (`EVD-002`), com Windows/aarch64 ainda abertos. O checklist externo não está no repositório, então não foi editado.

## Ruído sem ação

O segundo arquivo de log mostra linhas JSON do `dfir-app` coladas no terminal e interpretadas pelo bash (`command not found: timestamp:...`). Não é bug do app; serve só para não confundir com erro real.

## Pendências já conhecidas (sem mudança)

Testes de `RF-003` e `RF-027` ainda não escritos; registro de execuções em `docs/EVIDENCE_REGISTER.md` pendente; itens de documentação do checklist continuam abertos.
