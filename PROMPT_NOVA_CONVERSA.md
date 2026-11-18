<!-- doc-type: llm-rule | status: current -->
# Prompt de continuidade - DFIR/NIST / v6.5.1

Você é a LLM responsável por continuar este projeto. Faça a menor mudança suficiente e não transforme sugestões em requisitos.

## 1. Leitura por tarefa

Comece por `AGENTS.md` e `docs/DOCUMENT_AUTHORITY.md`. Depois leia apenas o que a tarefa exige:

- requisito/escopo: `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md`, `docs/DEMAND_TRACE.md`, `docs/PROJECT_SPEC_v6_5_1.md` e ADRs afetadas;
- estado atual: `docs/rtm.md`, nunca uma narrativa duplicada;
- trabalho restante: `docs/FASE1_BACKLOG.md` e tickets afetados;
- gates/evidências: `docs/RELEASE_GATE.md` e `docs/EVIDENCE_REGISTER.md`;
- operação/toolchain: `docs/OPERATION.md`, `docs/TOOLCHAINS.md`, `docs/SUPPLY_CHAIN.md` e manifests afetados.

O histórico completo não é contexto automático. Use `docs/archive/` e o histórico Git somente quando houver necessidade.

## 2. Arquitetura vigente

```text
Production Runtime
  Rust dfir-app (interface nativa eframe/egui/egui_extras)
  Rust dfir-engine

Build Toolchain
  Rust/Cargo

Engineering/QA Tooling
  CPython + Shell
```

- A GUI chama `src/service/` diretamente, no mesmo processo. Não há navegador, TypeScript, servidor HTTP ou porta de rede.
- App e engine comunicam-se por stdin/stdout + JSONL. O engine não acessa SQLite, usuários, NIST ou documentos finais.
- SQLite usa SQLx e SQL explícito. Migrations aplicadas são históricas.
- Python e Shell são ferramentas de QA, não backend de produção.

## 3. Regras de mudança

1. Identifique requisito, ADR, contrato e classe de runtime afetados.
2. Faça edições cirúrgicas. Não reescreva um arquivo inteiro para uma alteração local e preserve correções concorrentes.
3. Não introduza dependência, serviço, camada, API ou funcionalidade sem necessidade demonstrada.
4. Não altere arquitetura, transporte, persistência ou segurança sem verificar a decisão existente.
5. Atualize somente documentos afetados. Não duplique matrizes de estado nem narrações de rodada.
6. Faça commits pequenos e temáticos, sem dados de caso ou arquivos fora da tarefa.
7. Execute os gates aplicáveis e informe exatamente o que rodou e o que ficou pendente.

## 4. Invariantes de segurança

- Toda entrada externa é não confiável. Canonicalize caminhos contra a raiz confiável e rejeite symlink/reparse point no componente final.
- Não use shell intermediário em subprocessos de produção.
- Não execute o engine sem o processo supervisionado e os limites configurados.
- JSONL exige limite de linha e profundidade antes de desserializar, centralizados em `dfir-protocol`.
- Não registre secrets, nome de evidência ou conteúdo de eventos em logs/diagnósticos.
- Não oculte falhas com `continue`, defaults ou conversão silenciosa. `WAIVED` nunca significa `VERIFIED`.

## 5. Gates por mudança

```text
Documentação/contratos: just doc-gate
Código Rust:           just verify
Dependências:          just supply-chain
Fluxo com engine real: just e2e tests/fixtures/senai-demo.evtx
```

Antes de mudar dependências, use `cargo metadata --locked`, `cargo tree --locked -e features` e `cargo tree --locked -d`. Preserve `Cargo.lock` quando a tarefa não exigir atualização.

Windows, Linux aarch64 e incidente real exigem ambiente/evidência próprios. Não os declare verificados a partir de uma execução Linux x86_64 ou da fixture DEMO.

## 6. Estado e resposta

Estados de implementação: `PLANNED → IMPLEMENTED → INTEGRATED → VERIFIED → ACCEPTED`.
Gates: `PASS / WAIVED / PENDING / BLOCKED / STALE`.

Ao responder, informe arquivos alterados, comandos e resultados, evidências e pendências. Não declare conclusão apenas porque o código parece correto e não crie `EVD-*` sem execução registrada.

Toolchains de referência: `toolchains.toml`.
