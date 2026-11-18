<!-- doc-type: decision | status: accepted -->
# ADR-0010 — YARA-X e ruleset curado (F1-02)

## Status

ACEITA — 2026-08-25

## Contexto

F1-02 pede detecção de conteúdo de evidência com um ruleset YARA-X versionado, persistindo matches estruturados com origem, regra, versão e confiança/contexto, sem expor dados arbitrários. A seção 9 da spec canônica já decide o motor (YARA-X, seção 9.4 — não Hayabusa/Chainsaw, rejeitados na ADR-0002) e a política de ruleset (seção 9.1: **próprio e curado**, versionado no repositório, sem importar ruleset de terceiros como YARA-Forge por causa de licença GPL-3.0).

Duas perguntas de arquitetura precisavam de decisão: (1) onde a varredura roda (app ou engine), e (2) como o resultado chega ao modelo comum do F1-01 sem o protocolo JSONL (`dfir-protocol`) precisar conhecer o conceito de "match YARA".

## Decisão

- **Varredura roda no engine** (`dfir-engine`), não no app: AGENTS.md atribui "detecção" ao engine. O protocolo `Request` ganhou um campo `analysis_kind` (`EVTX_TIMELINE` | `YARA_SCAN`), permitindo que o mesmo par engine/app suporte os dois tipos de análise sem duplicar a máquina de estados de job (criação → PROCESSING → COMPLETED/FAILED, auditoria, custódia). `SCHEMA_VERSION` do protocolo subiu de "2" para "3" por ser um campo novo obrigatório.
- **YARA_SCAN não herda a restrição de formato EVTX**: YARA-X varre bytes, é agnóstico de formato. A checagem `evidence.format != "evtx"` em `analyze` (app) só se aplica quando `analysis_kind == EVTX_TIMELINE`.
- **O item de match é JSON genérico no protocolo** (como os itens de timeline EVTX já são) — `dfir-protocol` continua sem saber o que é um "match YARA"; quem interpreta o item é o lado app, exatamente como os itens EVTX viram `TimelineEvent` hoje. Isso evita acoplar o contrato JSONL compartilhado a um conceito de análise específico.
- **Ruleset embutido no binário** via `include_str!` (`engine/rules/dfir_mvp_ruleset.yar`), compilado em tempo de execução a cada processo do engine (o protocolo já é um request por processo — não há necessidade de cache entre requests). SHA-256 do ruleset é recalculado em tempo de execução, nunca hardcoded, para que o hash sempre reflita o conteúdo real compilado.
- **Ruleset 100% original** (4 regras: EICAR, PowerShell com comando codificado, palavras-chave de nota de resgate, comando conhecido do Mimikatz) — nenhuma regra de um ruleset de terceiros foi importada, evitando a questão de licença GPL-3.0 do YARA-Forge citada na própria seção 9.1.
- **Nenhum byte casado é exposto**: cada match carrega `pattern_id` + offset + comprimento, nunca `Match::data()` (os bytes reais). Isso satisfaz literalmente "sem expor dados arbitrários" do ticket.
- **Persistência em `analytical_results` (F1-01)** acontece no app, dentro da MESMA transação que marca o job como `COMPLETED` — job e resultados analíticos commitam ou revertem juntos. `AnalyticalResultType::YaraMatch` exige `evidence_id` (satisfeito, já que todo job tem uma evidência) e não exige `confidence` (a spec usa "confiança/contexto" para YARA, não "confiança obrigatória" como faz para NIST/ATT&CK/IOC).
- **Novo endpoint de leitura** `GET /api/evidence/{id}/analytical-results`, retornando o subconjunto seguro do modelo F1-01 (regra, versão, confiança, payload/provenance já sem bytes brutos por construção do engine).
- **`Cargo.toml` mantém as features default do `yara-x`** (não desabilitei módulos como `pe`/`elf`/`macho`/`crypto` que meu ruleset não usa). Decisão deliberada: `yara-x` embute `wasmtime`+`cranelift` incondicionalmente (a arquitetura do motor sempre compila condições para WASM — isso não é uma feature opcional), então a maior parte do peso da dependência já é inevitável dado que a spec já decidiu por YARA-X; tentar uma combinação mínima de features sem conseguir compilar neste ambiente (sem `cargo`) arriscava quebrar o primeiro build por uma causa evitável. Fica documentado como candidato a otimização, uma vez que o build possa ser verificado.

### Alternativas consideradas

- **Rodar a varredura no app em vez do engine.** Rejeitada: contraria a divisão de responsabilidades já normatizada em AGENTS.md (engine = parsing/detecção/normalização/correlação).
- **Adicionar um tipo `YaraMatch` ao `dfir-protocol`.** Rejeitada: o protocolo já usa itens JSON genéricos para EVTX; introduzir um tipo por analisador acoplaria o contrato compartilhado a cada nova capacidade analítica, quando F1-03/04/05 vão precisar do mesmo padrão.
- **Importar um ruleset público (YARA-Forge).** Rejeitada pela própria spec canônica (seção 9.1, GPL-3.0) antes mesmo desta ADR.
- **Cache do ruleset compilado entre múltiplos requests no mesmo processo (`OnceLock`).** Rejeitada por ora: o protocolo é request-por-processo (cada job spawna um novo `dfir-engine`), então não há um segundo request para reaproveitar o cache dentro do mesmo processo.

## Consequências

Positivas:
- F1-03/04/05 têm um padrão pronto para seguir: novo `analysis_kind` no protocolo (se precisarem rodar no engine) + persistência em `analytical_results` na transação de conclusão do job.
- Ruleset versionado e com proveniência auditável (hash recalculado, sem regra importada).

Negativas / dívida técnica assumida:
- `yara-x` com features default traz uma árvore de dependências pesada (`wasmtime`+`cranelift`, módulos `pe`/`elf`/`macho`/crypto não usados pelo ruleset atual) — candidato a redução de superfície uma vez que o build local confirme quais features são realmente necessárias.
- `read_result_items` (novo helper compartilhado com `job_results`) trunca a leitura do `results.jsonl` em `MAX_RESULT_LINES` linhas; para YARA_SCAN isso é reportado via um `_truncated` descartado no ponto de persistência — se um dia um ruleset gerar mais linhas do que esse limite, matches além dele não seriam persistidos silenciosamente truncados sem um sinal explícito no job. Não é esperado na prática (ruleset pequeno e curado), mas é uma lacuna conhecida.
  **Superseded por ADR-0020 (2026-10-04):** `MAX_RESULT_LINES`, `MAX_PERSISTED_YARA_MATCHES`, `max_scan_size` e o corte de 100 ocorrências por padrão foram removidos; ver a ADR-0020.

## Evidência de execução

Nenhum gate Rust (`cargo fmt`/`check`/`clippy`/`nextest`) foi executado neste ambiente (sem `cargo`/`rustc` instalados). A API real do crate `yara-x` 1.20.0 (`Compiler`/`Scanner`/`Rules`/`MetaValue`/`ScanError`/`CompileError`) foi conferida baixando e inspecionando o código-fonte publicado em crates.io antes de escrever `engine/src/yara_scan.rs`, mas isso não substitui compilação real. Um primeiro erro de build (lint `SqlSafeStr` do sqlx 0.9 em `analytical_results.rs`, ver `docs/FASE1_STATUS.md`) já foi reportado pelo operador e corrigido nesta mesma rodada — os próximos erros de `cargo build`/`cargo check` ainda não verificados devem ser reportados da mesma forma. **Passo obrigatório antes de qualquer gate `--locked`**: `Cargo.lock` não tem entradas para `yara-x`/`sha2`/suas dependências transitivas; rodar `cargo build` (sem `--locked`) uma vez primeiro para regenerar o lockfile.

## Adendo — 2026-09-21 — features do `yara-x` revisadas

A decisão original manteve as features padrão do `yara-x` "como decisão deliberada mas não verificada". A verificação foi feita (ver `docs/SUPPLY_CHAIN.md`, seção "Revisão do grafo do `yara-x 1.20.0`") e **revisa** aquela decisão:

- `engine/Cargo.toml` passa a declarar `yara-x = { version = "1.20.0", default-features = false, features = ["constant-folding", "exact-atoms", "fast-regexp"] }`.
- Motivo: o ruleset embutido não usa nenhum `import`; os módulos padrão (PE, ELF, Mach-O, .NET, hash, math etc.) e a criptografia que arrastam (`rsa` com RUSTSEC-2023-0071 sem correção, `x509-parser`, curvas elípticas) eram peso e superfície sem uso. O grafo caiu de 492 para 407 pacotes e o advisory do `rsa` desapareceu.
- Consequência: uma regra que use `import "pe"` (ou outro módulo) exige habilitar a feature do módulo e refazer a revisão de supply chain. O item YARA continua genérico (ADR-0010 original) — nada muda no formato do `YARA_MATCH`.
- Verificado por execução: o engine compila e passa em todos os testes com essa configuração, e um `YARA_SCAN` de ponta a ponta contra o binário `dfir-engine` real detectou a string de teste EICAR.
