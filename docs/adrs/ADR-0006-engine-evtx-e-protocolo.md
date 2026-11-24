<!-- doc-type: decision | status: accepted -->
# ADR-0006 — Engine fino sobre `evtx` e protocolo compartilhado

## Status
ATIVA — 2026-08-18

## Contexto
A implementação anterior continha protocolo duplicado, um desenho de serialização baseado em trait objects que não era compatível com `serde::Serialize` como `dyn`, e código de parsing que duplicava capacidade já fornecida por uma crate especializada.

## Decisão
- `evtx 0.12.2` com `default-features=false` é a implementação de parsing EVTX do MVP.
- O `dfir-engine` não reimplementa BinXML/EVTX.
- `dfir-protocol` é o único local dos tipos do protocolo JSONL compartilhados entre app e engine.
- As mensagens do protocolo usam tipos concretos `Serialize`/`Deserialize`; não usar `Box<dyn Serialize>`.
- Erros de registros individuais são emitidos como mensagens `ERROR` limitadas e não são colocados como objetos especiais dentro de `RESULT.items`.
- O supervisor encerra/recolhe o subprocesso e a tarefa de stderr em todo caminho de erro depois do spawn.
- A cadeia de auditoria da aplicação usa `BLAKE3-256-v2-JCS` para novos eventos, com o corpo do evento canonicalizado por RFC 8785 (JCS) e suporte de verificação para eventos `BLAKE3-256-v1`. Não há compatibilidade SHA-256 legada na cadeia porque esta árvore não possui eventos históricos de produção nesse formato.

Para evitar perda silenciosa de precisão, metadados com inteiros fora de `±2^53` devem ser representados como strings antes da canonicalização.

## Consequências
Positivas: menos código próprio, menos abstrações, contrato único, menor risco de vazamento de processos e separação mais clara entre parsing especializado e semântica DFIR.

Negativas: dependência adicional de uma crate externa e necessidade de validar sua versão/licença/grafo efetivos no gate Cargo.

## Reavaliação
Reabrir somente por bug demonstrado, requisito funcional, finding de segurança, incompatibilidade de licença ou benchmark que prove alternativa superior.

## Adenda — 2026-08-20 — limite de memória do engine

*(Sem conteúdo preenchido na revisão original. O cap de memória do processo do engine — `DFIR_ENGINE_MAX_MEMORY_BYTES`, via `RLIMIT_AS` em Unix — está documentado em `README.md` e no hardening do `CHANGELOG.md`.)*

**Nota — 2026-11-24**
Esta adenda ficou vazia na revisão original. As decisões posteriores sobre memória, contenção e sandbox do engine estão registradas nas seguintes ADRs:

- [**ADR-0024**](ADR-0024-contencao-do-engine-e-terminologia-de-sandbox.md) — contenção do engine (F1-22), limites de memória e terminologia de sandbox N1/N2, incluindo Landlock/seccomp no Linux e Job Object no Windows.
- [**ADR-0020**](ADR-0020-sem-limite-de-saida-e-truncamento.md) — moldura de 8 MiB e saída sem teto total, mantendo limites de memória e timeout como proteções explícitas.
- [**ADR-0021**](ADR-0021-timeline-no-banco-guarda-de-disco-e-avisos.md) — ajuste do teto de memória para arquivo mapeado, timeline persistida e guarda de espaço em disco.

Esta nota não altera retroativamente a decisão original da ADR-0006; apenas aponta para onde o tema foi tratado de forma completa.
