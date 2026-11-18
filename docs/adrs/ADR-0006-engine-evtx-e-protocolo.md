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

*(Sem conteúdo preenchido nesta revisão. O cap de memória do processo do engine — `DFIR_ENGINE_MAX_MEMORY_BYTES`, via `RLIMIT_AS` em Unix — está documentado em `README.md` e no hardening do `CHANGELOG.md`; esta adenda deveria registrar formalmente essa decisão.)*
