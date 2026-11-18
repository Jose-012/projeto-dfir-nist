<!-- doc-type: llm-rule | status: current -->
# AGENTS — Guia operacional para LLMs

## 1. Autoridade

A especificação canônica é `docs/PROJECT_SPEC_v6_5_1.md`. A cópia da raiz `nist-senai-proj_integrador_v6_5_1.md` deve permanecer byte-a-byte equivalente.

A autoridade por domínio está em `docs/DOCUMENT_AUTHORITY.md`.

Ordem de precedência:

1. requisitos explícitos da demanda (`docs/source/DEMANDA_ORIGINAL_SENAI_12412.md`; rastreabilidade em `docs/DEMAND_TRACE.md`);
2. corpo normativo da especificação;
3. decisões/ADRs/RTM/status e contratos;
4. índices/XML/prompt derivados;
5. hipóteses, exemplos e sugestões.

Não transforme hipótese, exemplo, inferência ou sugestão em requisito.

Pedidos do operador do projeto que mudam tecnologia ou escopo (ex.: ADR-0017, interface nativa; ADR-0018, poda de escopo; ADR-0019 a ADR-0022, CLI headless, sem limite de saída, timeline no banco/guarda de disco e observabilidade; ADR-0023, tipos analíticos novos e plano de conclusão do protótipo; ADR-0024, contenção do engine e terminologia de sandbox; ADR-0025, evento normalizado, detecção por regras com confirmação e escopo EVTX; ADR-0026, cópia verificada do banco antes de migration; ADR-0027, harness de agente de desenvolvimento; ADR-0028, SBOM fora do escopo) são **decisões de arquitetura/escopo** (nível 3), registradas por ADR com atualização da especificação. Não são "requisitos explícitos da demanda" e não invocam a precedência nº 1. Só o texto da demanda original ocupa o nível 1.

## 2. Classes de runtime

O projeto distingue:

```text
Production Runtime
  Rust dfir-app (com interface nativa eframe/egui/egui_extras) + Rust dfir-engine + ferramentas indispensáveis

Build Toolchain
  Rust/Cargo (nenhum toolchain de frontend — ADR-0017 removeu Node.js/npm/esbuild/vitest do projeto)

Engineering/QA Tooling
  CPython + Shell + fixtures + benchmarks + scripts
```

Regra fundamental: **minimizar o runtime de produção; não exigir uma única linguagem em todas as fases de desenvolvimento**.

Node, Bun e Deno não pertencem à imagem/runtime de produção nem ao build toolchain: o frontend que os usava foi removido (ADR-0017 — interface nativa eframe/egui substitui o par Axum + `frontend/` TypeScript; ADR-0009, sobre Bun→Node, fica só de valor histórico). Python também não pertence ao runtime por padrão.

## 3. Interface

A interface é nativa (eframe/egui/egui_extras, `src/gui/`), parte do mesmo binário `dfir-app`. Não há navegador, HTML, TypeScript, servidor HTTP nem porta de rede — nem em loopback (ADR-0017).

- `src/gui/` chama `src/service/` diretamente, no mesmo processo — sem serialização JSON no meio, sem rota.
- A UI (egui) roda na thread principal e é síncrona; toda chamada assíncrona (SQLite, subprocesso do engine) acontece em uma thread de backend dedicada (`src/gui/backend.rs`), nunca na thread da UI. Ver `docs/NATIVE_UI_ARCHITECTURE.md`.
- Novas dependências de UI (crates `eframe`/`egui`/`egui_extras`/outras) entram só mediante necessidade demonstrada e nova validação de supply chain/offline — mesma régua que valia para bibliotecas de frontend antes desta migração.
- Diálogos nativos de arquivo (`rfd`) são o único ponto de entrada de caminho escolhido interativamente pelo operador; a aplicação continua validando/canonicalizando esse caminho como qualquer outra entrada não confiável.

## 4. Backend e engine

`dfir-app` contém a interface, domínio, persistência, auditoria, fila, relatórios e supervisão do engine.

`dfir-engine` é processo separado e trata parsing/detecção/normalização/correlação. Ele não conhece SQLite, identidade de usuário, sessão, NIST ou geração final de documentos.

O transporte de produção é stdin/stdout + JSONL. Não introduzir socket TCP/Unix ou HTTP do engine sem ADR.

## 5. Python/Shell

Python/Shell são ferramentas de engenharia/QA, não backend do produto.

Podem fazer:

- fixtures;
- validação;
- fuzzing/orquestração;
- benchmarks;
- verificações de release;
- testes de sandbox;
- inspeção de artefatos.

Não podem presumir acesso ao banco, secrets ou infraestrutura privilegiada. Transformar um analisador Python em worker de produção exige ADR, threat model, testes, benchmark e revisão de supply chain.

## 6. Segurança

Toda entrada externa é não confiável.

Nunca:

- usar shell intermediário em subprocessos de produção;
- aceitar caminho físico arbitrário sem validar/canonicalizar contra a raiz confiável, mesmo quando escolhido pelo próprio operador via diálogo nativo de arquivo;
- executar engine fora do processo supervisionado e dos limites de execução como fallback;
- registrar secrets/tokens/senhas;
- ocultar falhas relevantes com `continue`, default ou conversão silenciosa;
- confundir `WAIVED` com `VERIFIED`.

Em caminhos de entrada e processamento não confiável, evitar `unwrap()`/`expect()` sem justificativa verificável.

JSONL é uma fronteira de entrada não confiável: verificar tamanho máximo e profundidade estrutural antes de desserializar. O limite de profundidade deve permanecer centralizado no crate `dfir-protocol` e ser aplicado tanto pelo app quanto pelo engine.

Caminhos resolvidos a partir de referências lógicas (`evidence/...`) devem ser canonicalizados, validados contra a raiz confiável, e ter o componente final rejeitado se for symlink/reparse point (Windows: verificar o bit de reparse point genérico, não só `IO_REPARSE_TAG_SYMLINK`, para cobrir junctions). Essa checagem não elimina o TOCTOU entre canonicalize e o `open` real — apenas o estreita — e isso é um risco residual aceito enquanto o modelo de ameaça for o operador local único (ADR-0007), não um processo hostil co-residente ativamente correndo contra o disco.

## 7. Persistência

SQLx + SQLite + SQL explícito. Repositories finos organizam persistência, mas não escondem invariantes de segurança, autorização ou transações críticas.

Não introduzir ORM sem ADR e evidência de que reduz risco/complexidade sem degradar rastreabilidade/offline.

Migrations são históricas; não editar uma migration aplicada para alterar seu significado.

## 8. Dependências

O `Cargo.lock` e `cargo metadata --locked` são autoridade sobre versões resolvidas.

Antes de mudar dependência, verificar:

```text
cargo metadata --locked
cargo tree --locked -e features
cargo tree --locked -d
cargo deny --workspace check
cargo audit
```

`cargo vet` está dispensado no MVP (ADR-0018 D-2); permanecem `cargo deny --workspace check` e `cargo audit` (confirmado pelo operador em 2026-11-04). Gap origem: INC-04.

Depois:

```text
cargo fmt --all -- --check
cargo check --workspace --locked --all-targets
cargo clippy --workspace --locked --all-targets --all-features -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
```

> Gap origem: INC-02 (2026-11-04). `--workspace` é necessário porque a raiz é um package e o workspace (`members = [".", "engine", "protocol"]`) não declara `default-members`; sem ele, os comandos só cobrem `dfir-app`. Ver `docs/RELEASE_GATE.md`.

Se Cargo/Python não estiver disponível, não declarar a respectiva execução como verificada.

## 9. Processo de alteração

1. identificar requisito/decisão/contrato;
2. determinar classe de runtime afetada;
3. fazer a menor mudança estrutural suficiente;
4. atualizar testes;
5. atualizar documentação e RTM;
6. atualizar ADR se decisão mudar;
7. executar gates disponíveis;
8. invalidar evidências `PASS` que ficaram `STALE`;
9. registrar fatos como `REPO-FACT`, `EXTERNAL-FACT`, `ASSUMPTION`, `INFERENCE` ou `PENDING-HUMAN` quando apropriado;
10. relatar exatamente o que foi executado;
11. separar commits pequenos por mudança coerente, sem dados de caso, artefatos operacionais ou arquivos fora da tarefa.

## 10. Estados

```text
PLANNED → IMPLEMENTED → INTEGRATED → VERIFIED → ACCEPTED
```

Um gate usa:

```text
PASS / WAIVED / PENDING / BLOCKED / STALE
```

`WAIVED` é exceção de governança, não prova de verificação.

IDs normativos (`RF-*`, `D*`, `LLM-*`, `CA-*`, `TEST-*`, `GATE-*`, `ADR-*`, `EVD-*`, `WVR-*`) nunca são reutilizados. Mudanças de decisão usam `supersedes`.

## 11. Documentação obrigatória

Atualize somente os documentos afetados pela mudança:

- requisitos/arquitetura: `docs/PROJECT_SPEC_v6_5_1.md`, cópia raiz equivalente, `docs/DEMAND_TRACE.md` e `docs/adrs/`;
- estado/rastreabilidade: `docs/rtm.md`, `docs/FASE1_BACKLOG.md` e tickets afetados;
- gates/evidências: `docs/RELEASE_GATE.md`, `docs/EVIDENCE_REGISTER.md` e `docs/evidence/`;
- uso/operação: `README.md`, `docs/OPERATION.md`, `docs/DEMO.md`, `docs/TOOLCHAINS.md`, `docs/SUPPLY_CHAIN.md`;
- contratos específicos: `docs/NATIVE_UI_ARCHITECTURE.md`, `docs/DETECTION_RULES.md`, `docs/API.md`;
- histórico: `docs/FASE1_STATUS.md` ou `CHANGELOG.md`, sem copiar matriz de estado nem repetir toda a narrativa;
- regras de agente: `AGENTS.md`, `CONVENTIONS.md`, `PROMPT_NOVA_CONVERSA.md`, `docs/DOCUMENT_AUTHORITY.md`.

`docs/FASE0_STATUS.md`, `docs/DEFINITION_OF_READY.md` e `RELATORIO_ALTERACOES_*` são históricos; só mudam para correção factual ou atualização do histórico. O Git log registra as rodadas novas. Não crie um relatório permanente para cada alteração de rotina.

> Gap origem: AUS-01/INC-14; simplificação de documentação em 2026-11-18.

## 12. Regra final

A LLM nunca deve declarar “concluído” porque o código parece correto. Conclusão exige evidência correspondente ao nível afirmado.


Toolchains de referência: `toolchains.toml`.


## 13. Hardening preventivo pré-congelamento
- O processamento de evidência deve falhar fechado quando o processo do engine não puder ser criado ou supervisionado com os limites configurados.
- A interface nunca deve reduzir uma falha específica (validação rejeitada pela persistência, engine indisponível, timeout) a uma mensagem genérica quando houver diagnóstico útil disponível — ver `AppError::user_message()`.

## 14. Logs e dados de evidência (ADR-0022)

- Logs e `dfir-app diagnostics` podem conter ids, tamanhos, contagens, durações e códigos de erro. **Nunca** devem conter o nome do arquivo de evidência nem conteúdo de eventos (há teste que verifica o diagnóstico; revise novos `tracing::*` com esse critério).
- Não há telemetria remota; qualquer proposta de enviar dados para fora exige ADR e passa pela restrição RB-006 da demanda.

## 15. Ferramentas preferidas do agente (economia de contexto)

Atualizado em 2026-11-18 (verificado no host: `rtk 0.49.0`, `rg`, `ast-grep 0.51.2`/`sg` (sg depreciado), `fd`, `zoxide`, `bat` e `eza` instalados; pager `delta`, `jcodemunch` não instalados).

**Busca e leitura de código — use as ferramentas nativas, não o shell:**

- Busque código com `agentgrep` (modo `grep`/`find`/`outline`/`trace`), **nunca** `grep -r`/`rg` via bash para explorar o workspace. O agentgrep inclui estrutura do arquivo (funções e suas localizações) e poda resultados já vistos. Para busca estrutural por AST use `ast-grep` (`sg` está depreciado; use `ast-grep`) via bash, com `--lang rust` e pattern, em vez de regex frágil.
- Leia arquivos com `read` (com `start_line`/`limit`), nunca `cat`/`head`/`tail` via bash. Edite com `edit`/`apply_patch`/`write`, nunca `sed`/`perl`/scripts Python via bash.
- Liste arquivos com `agentgrep` modo `find`, nunca `ls -R`/`find` amplos via bash. Para listar diretório pontual em bash, prefira `fd` (ou `eza -T` para árvore) sobre `ls`/`find`.

**Shell (quando inevitável):**

- Agrupe comandos independentes em uma única chamada `batch` ou em um único comando composto (`&&`), em vez de várias chamadas bash sequenciais.
- Prefira `rtk <cmd>` (ex.: `rtk cargo test`, `rtk git ...`, `rtk grep ...`, `rtk test`, `rtk err`) para saída condensada de comandos barulhentos; se a saída filtrada ficar irrecuperável, re rode com `rtk proxy <cmd>` (o truncamento informa o caminho de recuperação). `rtk init` já está refletido no `CLAUDE.md` deste projeto.
- Navegação: `zoxide` (`z <dir>`) em vez de `cd` com caminhos longos; arquivos grandes legíveis com `bat --paging=never` (ou `bat -p`) em vez de `cat` quando precisar paginar; diffs com `git diff`. Não presuma que o pager `delta` esteja instalado: `/usr/bin/git-delta` pode ser o comando homônimo do git-extras.
- Arquivos temporários de trabalho vão em `$JCODE_SCRATCH_DIR`, nunca em `/tmp`.
- Nunca use shell intermediário para subprocessos de produção nem registre secrets; as regras de segurança do §6 prevalecem sobre qualquer preferência de ferramenta.

