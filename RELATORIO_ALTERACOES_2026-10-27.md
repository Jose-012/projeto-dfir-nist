<!-- doc-type: log | status: current -->
# Relatório de alterações — 2026-10-27

> **Nota (2026-11-04) — Gap origem: EVI-02** (cabeçalho `doc-type`: AUS-08). A linha "Documentação" abaixo declara RELEASE_GATE, NATIVE_UI_ARCHITECTURE e tickets como atualizados. Isso não se sustenta integralmente: a auditoria de 2026-11-04 encontrou `INC-01` (`docs/RELEASE_GATE.md:57` ainda dava a cópia automática como pendente; corrigido em 2026-11-04), `INC-10` (`docs/NATIVE_UI_ARCHITECTURE.md` sem quatro módulos de `src/gui/` e sem os módulos novos de `src/service/`, e com uma limitação já superada; **ainda aberto**) e `F1-12`, `F1-14`, `F1-15` (texto de status de ticket defasado; reconciliado por notas datadas em 2026-11-04). O texto original permanece.

Pedido: concluir as etapas da análise (contenção do engine, terminologia de sandbox, lacunas da demanda original), lendo toda a documentação. Decisões: `docs/adrs/ADR-0024`, `ADR-0025`, `ADR-0026`.

## O que mudou no veredito sobre a demanda

| Item da demanda | Antes (v6.5.0) | Agora |
|---|---|---|
| Vetor inicial, movimentação lateral, escalonamento de privilégios | registro manual do analista | **detecção por regras sobre o EVTX, com confirmação do analista** (19 regras, `docs/DETECTION_RULES.md`) |
| Tentativa de exfiltração | registro manual | regras para a **tentativa** (ferramenta de envio, conexão de saída, compactação). **Não confirma a transferência sem log de rede** |
| Fontes de rede, SIEM, dispositivos de segurança | `PARCIAL`/`PLANEJADO` | **`ADIADO` por decisão sua** (F1-27). Ver "Decisões para você vetar" |
| Sandbox | processo separado + `RLIMIT_AS` (só Unix) | Linux: `no_new_privs` + Landlock + seccomp, **provados com o engine real**. Windows: Job Object, **só compilado** |
| Migration | backup manual | **cópia verificada automática** antes de migrar (RF-032) |

`COBERTO` continua significando "existe funcionalidade", não "validado". As regras foram vistas só no `security.evtx` real (que não contém ataque) e em eventos sintéticos, **nunca num incidente real**.

## Verificação (rustc/cargo 1.97.1, o toolchain pinado, Linux x86_64, `--locked`)

| Gate | Resultado |
|---|---|
| `cargo test --workspace` | app **288** (eram 240; 1 ignorado como antes), engine **83** + **7** de integração do sandbox (eram 43), protocolo **26** (eram 11) |
| `cargo clippy --workspace --all-targets -- -D warnings` | limpo (3 avisos antigos de `assertions_on_constants` no protocolo, que o 1.97.1 acusa, foram corrigidos) |
| `cargo fmt --all -- --check` | limpo |
| `scripts/e2e_headless.py`, engine e `security.evtx` reais | **110/110** (eram 79) |
| `cargo check --target x86_64-pc-windows-gnu -p dfir-engine` | compila, sem avisos. **Nada foi executado em Windows** |
| `scripts/doc_gate.py`, `scripts/validate_fase0.py` | PASS; 0 erros, 0 avisos |

O pacote de 2026-10-23 **já passava** no 1.97.1 (app 240, engine 43, protocolo 11); a documentação dizia "nada verificado no pinado" porque ninguém tinha tentado. O toolchain veio do conda-forge (`docs/TOOLCHAINS.md`), sem cópias locais de crates.

## Defeitos reais achados ao rodar o caminho inteiro

Nenhum aparecia nos testes anteriores:

1. **O engine entrava em pânico sob seccomp** ao criar o runtime do tokio (driver de sinais usa `socketpair`). Só acontece na build do workspace; `cargo test -p dfir-engine` não reproduz. `socketpair` foi liberado de propósito (ADR-0024 D-3).
2. **`analysis_jobs.kind` tinha `CHECK` com dois tipos**: a detecção falhava ao gravar o job. A migration 0012 recria a tabela, com teste de preservação.
3. **Filtro de timeline inválido derrubava a tela** e escondia a própria caixa. A interface agora valida antes de enviar.
4. **Evidência descartada** aparecia na seção nova do relatório como "nenhuma evidência citada" e "nenhum candidato" (candidatos apagados com ela). Agora diz "evidência descartada".
5. `verify_published` abria o arquivo adulterado antes de conferir o hash.

Uma limitação que **não** é defeito: o guarda de disco recusou a importação neste ambiente (686 MiB livres, margem de 1 GiB) — comportamento correto.

## O que foi feito

- **F1-22 (ADR-0024).** `env_clear` com ambiente mínimo; **arquivo único imposto** (`DFIR_ENGINE_INPUT_FILE` era definida e nunca lida); `kill_on_drop`; sandbox aplicado pelo engine **antes de ler a entrada**; política `DFIR_ENGINE_SANDBOX` (`required` no Linux, valor inválido recusa a partida); estado no `PROGRESS`, na auditoria e como aviso `SANDBOX_NOT_FULLY_APPLIED`. Prova: `engine --sandbox-selftest` tenta, de dentro do processo contido, ler outro arquivo, criar arquivo, abrir socket, `execve`, `ptrace` (todos negados) e ler o arquivo do job (permitido); há um controle (com `off` as mesmas tentativas funcionam).
- **Terminologia.** "Isolamento" (processo separado, ambiente reduzido, arquivo único) vs "sandbox" (contenção do SO) vs "máquina dedicada". `RF-033` e `RF-036` reescritos; README, CONVENTIONS e o prompt de continuidade alinhados. A demanda **não** exige sandbox de SO.
- **F1-26 (ADR-0025).** Campos estruturais opcionais no evento (esquema 4, migration 0012); o app recalcula os campos de resultados antigos; filtro `campo:valor` na timeline e em `dfir-app results --filter`.
- **F1-28 (ADR-0025).** Candidatos pendentes com ATT&CK sugerida, eventos de origem, IOCs e **o que a regra não prova**; **confirmar** cria mapeamento + achado + IOCs (retomável se falhar no meio), **rejeitar exige motivo**; decisão final protegida por gatilhos; aba "Detecções" na tela de evidência; `dfir-app detect | candidates | candidate-confirm | candidate-reject`; Entregável 1 §4.3 declara confirmados, rejeitados e **pendentes** ("não avaliados e não constam como achado").
- **RF-032 (ADR-0026).** `VACUUM INTO` + SHA-256 + `integrity_check` + `foreign_key_check` + contagem de linhas por tabela, antes de migrar; falha fechada; `dfir-app backup-verify`.
- **Caracteres de direção de texto.** Viram o marcador visível `⟦U+202E⟧` na timeline, nos candidatos e nos cinco documentos (o texto original fica intacto).
- **Documentação.** Especificação (nota 8; as duas cópias idênticas), RTM, DEMAND_TRACE, FASE1_STATUS, FASE1_BACKLOG, tickets, README, CHANGELOG, OPERATION (§3 a §3C), DEMO, RELEASE_GATE, SUPPLY_CHAIN, NATIVE_UI_ARCHITECTURE, API, TOOLCHAINS, fixtures; nova `docs/DETECTION_RULES.md`, amarrada ao catálogo por um teste.
- **Cargo.lock:** só duas arestas novas no engine (`libc`, `windows-sys 0.61.2`). Nenhum crate novo.

## Decisões para você vetar

1. **F1-27 (importador CSV/JSONL) adiado e `RF-049` fora do MVP.** Você já tinha mandado ignorar formatos além de EVTX (ADR-0023); formalizei, e as linhas de rede/SIEM/dispositivos viraram `ADIADO` (não `PARCIAL`). Consequência declarada: a ferramenta **não analisa** rede, SIEM nem dispositivos de segurança. Se quiser reverter, F1-27 é independente e as regras passam a valer sobre ele.
2. **`required` como padrão no Linux.** Sem Landlock (kernel < 5.13) ou em **aarch64** (seccomp só x86_64) o engine **recusa analisar** até você escolher `best-effort`. É o que RF-036 pede; avise se prefere o contrário.
3. **`socketpair` liberado no seccomp** (par anônimo; `socket`, `connect`, `bind` seguem negados).
4. **Confiança máxima `MEDIUM`** para regra automática, e `R-PRIV-001` com gravidade `LOW` (todo administrador recebe esses privilégios).

## O que não foi feito / não está verificado

- **Windows:** o Job Object nunca foi executado; **não há confinamento de arquivo nem de rede** (exigiria AppContainer ou token restrito, não feito). Todo job lá registra `SANDBOX_NOT_FULLY_APPLIED`. Rode `dfir-app engine-smoke` e uma análise lá antes de confiar.
- **Linux aarch64:** não testado.
- **Incidente real:** as regras nunca rodaram sobre um. As regras de rajada, processo, serviço e log apagado **não são exercitadas pelo `security.evtx`** (ele não tem 4625, 4688, 7045 nem 1102); têm fixtures sintéticas.
- **GUI:** só `egui_kittest`; **nenhuma janela real foi aberta**.
- **`cargo deny`/`cargo audit`:** não reexecutados (F1-15). Como não há crate novo, é inferência que o resultado anterior vale, não medição.
- **CI (F1-14)**, fixtures EVTX versionadas (F1-16), vínculo evidência↔incidente (F1-19), custódia visível (F1-24): continuam pendentes.
- **Limites do método (repetidos nos documentos):** a regra só enxerga o que o log registrou (auditoria de processo, compartilhamento e plataforma de filtragem precisam estar ligadas no Windows); a ausência de candidato não prova ausência de ataque; casamento por texto não pega ferramenta renomeada; o `clone` com flags de namespace não é bloqueado (o seccomp clássico não lê `clone3`).
- Nada aqui está `ACCEPTED`: os itens ficam `IMPLEMENTED / PENDING-HUMAN`.
