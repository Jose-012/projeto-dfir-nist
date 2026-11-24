<!-- doc-type: decision | status: accepted -->
# ADR-0031 — Propriedade de jobs por lock exclusivo e recuperação multiinstância

## Status

ACEITA — 2026-11-24, dentro da missão explícita do operador de concluir `docs/FINAL_ANALYSIS_CHECKLIST.md`. Supersede ADR-0020, item 6 da decisão, somente quanto à identificação de órfãos, e a nota "Recovery ... continua inalterado" da ADR-0030 D-4. O processo separado, o cancelamento persistido e a saída sem teto total permanecem.

## Contexto

`recover_orphaned_jobs` selecionava todos os jobs `PENDING`/`PROCESSING` e cada GUI executava essa recuperação na partida. Assim, uma segunda instância podia marcar como `FAILED/ORPHANED` um job que ainda estava sendo executado pela primeira.

A regressão `second_instance_recovery_preserves_live_job` reproduziu esse comportamento em Linux x86_64 com um engine de teste realmente vivo: a segunda instância retornou `recovered = 1`, em vez de preservar `(0, "PROCESSING")`. O teste recolheu o processo fake antes da assertiva final. A correção precisa também preservar a recuperação de jobs cujo dono morreu, o cancelamento entre instâncias e a auditoria exatamente uma vez.

## Decisão

### D-1 — Um lock de propriedade por job

Cada análise cria `staging/analysis/<job_id>/owner.lock` e adquire um lock exclusivo com `std::fs::File::try_lock` **antes** de publicar a linha `PROCESSING`. O handle permanece aberto durante execução, indexação, escrita de resultados e auditoria. Fechar o handle, inclusive por término abrupto do processo, libera o lock pelo sistema operacional.

A API estável desde Rust 1.89 corresponde a `flock` no Unix e `LockFileEx` no Windows. Não há migration, dependência nova, PID, heartbeat ou lease temporal. O arquivo contém somente o mecanismo de lock, sem nome de evidência ou conteúdo de eventos.

### D-2 — Recuperar somente quando a propriedade puder ser adquirida

Na partida, cada job ativo passa por estas regras:

- `TryLockError::WouldBlock`: outro handle ainda é dono; o job permanece `PROCESSING` e nenhuma auditoria de órfão é escrita;
- lock adquirido: o dono anterior não mantém mais o job; a recuperação conserva seu próprio handle até o commit;
- `owner.lock` ausente: job legado sem o mecanismo novo; a recuperação continua permitida, mas usa a mesma transação condicional abaixo;
- outro erro de abertura/lock: a recuperação falha e não marca o job.

IDs persistidos usados para formar o caminho são restritos a caracteres alfanuméricos ASCII, hífen e sublinhado, evitando traversal.

### D-3 — Transição e auditoria são atômicas

A recuperação usa `BEGIN IMMEDIATE` e só atualiza uma linha ainda `PENDING`/`PROCESSING` com `cancellation_requested_at IS NULL`. `ANALYSIS_JOB_ORPHANED` é escrito na mesma transação. Zero linhas atualizadas causa rollback e nenhum evento. Isso serializa duas recuperações legadas e impede que um cancelamento concorrente se converta em órfão.

### D-4 — Compatibilidade delimitada

Jobs escritos antes desta ADR não possuem lock e seguem recuperáveis. Uma versão anterior não protege seus jobs contra a recuperação da versão nova; portanto, versões mistas não devem executar simultaneamente sobre o mesmo `data_root`. Instâncias da versão nova podem coexistir e o cancelamento persistido entre elas continua suportado. Não se impõe instância única.

## Alternativas consideradas

- **PID com verificação periódica:** rejeitado por reutilização de PID e necessidade de APIs específicas por plataforma.
- **Lease/heartbeat com expiração:** rejeitado porque um job legítimo pode ficar pausado além do prazo e porque escolher esse prazo seria arbitrário.
- **Instância única:** rejeitado pelo checklist e pela existência do cancelamento entre instâncias.
- **Somente transação SQLite:** necessária para idempotência, mas insuficiente para distinguir um job vivo de um órfão sem o lock.

## Consequências

A partida de outra instância não altera um job vivo da versão nova. Um job sem dono vivo vira `FAILED/ORPHANED` no máximo uma vez, com auditoria íntegra. Resultados concluídos, índices e cancelamentos mantêm seus contratos. O comportamento Windows é definido pela API multiplataforma, mas sua execução nativa continua pendente de ambiente.

## Supersedes

- ADR-0020, item 6 da decisão, somente a identificação e recuperação de jobs órfãos.
- ADR-0030 D-4, somente a frase que mantinha a recuperação anterior inalterada.

## Superseded-by

Nenhum até esta decisão.

## Evidência

- Before: `validation/task-001-before-qualified.txt` no run overnight, `cargo test --locked -p dfir-app second_instance_recovery_preserves_live_job -- --nocapture`, exit 101, `recovered = 1` em vez de zero.
- After focado: `validation/task-001-after-focused.txt`, mesmo teste, 1 passed, exit 0.
- Recuperação: `validation/task-001-recovery-suite.txt`, 2 passed, exit 0.
- Cancelamento entre instâncias: `validation/task-001-cancellation.txt`, exit 0.
- Gates integrados serão registrados somente após execução e entrada em `docs/EVIDENCE_REGISTER.md`; esta ADR não antecipa novo `PASS`.
