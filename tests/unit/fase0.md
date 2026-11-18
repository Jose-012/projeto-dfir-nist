# Fase 0 - testes unitários e invariantes

- limites de configuração do engine;
- canonicalização de metadata;
- encoding inequívoco da audit chain;
- verificação de cadeia BLAKE3-v2-JCS e compatibilidade com BLAKE3-v1;
- serialização de writers com `BEGIN IMMEDIATE`;
- parsing defensivo da timeline na interface nativa (`src/gui/evidence.rs`).
## Timeline EVTX — timestamp semântico

O timestamp da timeline deve vir de `Event/System/TimeCreated/#attributes/SystemTime` quando presente e válido. O `EvtxRecord.timestamp` é apenas fallback para casos em que o timestamp semântico do evento não esteja disponível. Isso evita exibir `1601-01-01T00:00:00Z` quando o cabeçalho do registro estiver zerado.

