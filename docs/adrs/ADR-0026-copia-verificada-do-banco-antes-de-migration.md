<!-- doc-type: decision | status: accepted -->
# ADR-0026 — Cópia verificada do banco antes de migration (RF-032)

## Status

ACEITA — 2026-10-27. Fecha o pendente nº 1 do relatório de 2026-10-23 ("cópia e verificação automáticas do banco antes de migration").

## Contexto

As migrations 0010, 0011 e 0012 **recriam tabelas** (`analytical_results`, `analysis_jobs`). Uma falha no meio, ou um defeito na própria migration, custaria o banco de um caso. Até aqui a proteção era um backup **manual** descrito em `docs/OPERATION.md`, que depende de o operador lembrar.

## Decisão

Antes de aplicar qualquer migration pendente sobre um banco **que já tem dados**, a aplicação, na partida da CLI e da GUI:

1. cria uma cópia consistente com `VACUUM INTO` (inclui o conteúdo do WAL) em `<dados>/backups/pre-migration-<UTC>-v<de>-to-v<para>/dfir.sqlite`;
2. calcula o SHA-256 da cópia;
3. **verifica** a cópia aberta somente leitura: `PRAGMA integrity_check`, `PRAGMA foreign_key_check`, as mesmas migrations aplicadas e a **mesma contagem de linhas de cada tabela** do banco de origem;
4. grava `manifest.json` (versões, SHA-256, tamanho, contagens, resultado da verificação) e só então publica a pasta (escreve em `.partial` e renomeia);
5. aplica as migrations; e registra `DB_PRE_MIGRATION_BACKUP` na cadeia de auditoria (nome da pasta, versões, hash, tamanho; **sem caminho absoluto**).

**Falha fechada:** se a cópia ou a verificação falhar (inclusive por falta de espaço: o tamanho do banco mais `DFIR_MIN_FREE_DISK_BYTES`), a cópia parcial é descartada e **a migration não é aplicada**. Banco novo (nunca migrado) e banco já atualizado não geram cópia. A pasta `backups/` fica protegida contra ser escolhida como destino de exportação, e as permissões são `0700`/`0600` em Unix.

`dfir-app backup-verify <pasta>` reverifica uma cópia depois, de forma independente (o hash vem antes de abrir o arquivo, então um arquivo adulterado nem chega ao SQLite).

## O que isto não faz

- Não copia `evidence/` (migrations não a tocam; o backup da pasta de dados inteira de `docs/OPERATION.md` continua valendo para o resto).
- Não apaga backups antigos: eles acumulam; limpar é decisão do operador.
- Não restaura sozinho: restaurar é copiar `dfir.sqlite` de volta com o app fechado (passos em `docs/OPERATION.md`).

## Consequências

- Bug encontrado ao testar: `verify_published` abria o arquivo adulterado antes de conferir o hash e o SQLite reclamava do esquema; a ordem foi corrigida e o teste cobre.
- A migration 0012 tem teste que a aplica sobre um banco v11 com jobs reais e confere linhas, tipo, avisos, índices, chave estrangeira e o índice de timeline invalidado.
