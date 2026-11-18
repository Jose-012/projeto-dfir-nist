<!-- doc-type: decision | status: accepted -->
# ADR-0021 — Timeline no banco, guarda de espaço em disco e avisos do parser

## Status

ACEITA — 2026-10-11, a pedido explícito do operador ("resolva todos os riscos antes de eu testar a build"). Decisão de arquitetura (nível 3); não altera RF. Fecha os riscos que a ADR-0020 deixou abertos (memória, disco, EVTX truncado, arquivo grande).

## Contexto

A ADR-0020 removeu os limites de saída e o truncamento, e registrou quatro riscos: (1) a timeline era lida inteira para a memória; (2) sem teto de saída, só o tempo do engine freava o uso de disco; (3) um EVTX truncado terminava `COMPLETED` sem aviso (AUD-36); (4) itens não verificados (GUI renderizada, arquivo grande no Linux, supply chain da UI).

## Decisão

1. **Timeline persistida e paginada (F1-18).** Migration `0009`: tabela `timeline_events` (chave primária `(job_id, sort_ts, ord)`, `WITHOUT ROWID`, ou seja, a chave é a ordem cronológica) e `timeline_index`. O índice é construído em fluxo ao concluir o job EVTX (memória constante) e, para jobs antigos, na primeira consulta. A GUI e a CLI leem **páginas** (`timeline_page`: offset, limite, filtro, total opcional); o filtro (título, descrição, provedor, canal; ASCII sem diferenciar maiúsculas; `%` e `_` literais) roda no SQLite. A GUI guarda no máximo 48 páginas de 200 linhas, pede só as que faltam e mostra a timeline do último job ao selecionar a evidência, sem reanalisar (antes só aparecia logo após analisar na mesma sessão). Itens sem timestamp válido deixam de sumir em silêncio: viram o aviso `TIMELINE_ITEMS_SKIPPED`. O índice sai junto com a evidência no descarte (RB-006).
2. **Guarda de espaço em disco (F1-10).** `DFIR_MIN_FREE_DISK_BYTES` (padrão 1 GiB, 0 desliga, máx. 1 TiB): a importação exige tamanho do arquivo + margem; a análise não começa abaixo da margem (falha antes de criar o job); durante a análise o app confere o volume a cada segundo e interrompe o engine com `DISK_SPACE_LOW`. Implementada para Unix (`statvfs`); em outras plataformas a medição devolve "desconhecido" e **não bloqueia** (a guarda fica desligada ali).
3. **Avisos não fatais do parser (F1-17, AUD-04/AUD-36).** O engine emite `EVTX_TRUNCATED` (arquivo que não é um número inteiro de chunks ou com menos chunks completos que o cabeçalho declara) e `EVTX_RECORDS_MISSING` (cabeçalho indica registros além do último lido). São heurísticas: cabeçalho de log "sujo" fica defasado, o que só torna o aviso menos sensível, nunca gera falso positivo (verificado no EVTX real íntegro: sem avisos). O app agrega **todas** as mensagens não fatais do engine (`PARSER_RECORD_ERROR` inclusive, antes descartadas) em `analysis_jobs.warning_count/warnings`; a GUI exibe os avisos em destaque e a CLI os devolve no JSON.
4. **Arquivo grande no Linux (F1-20).** `RLIMIT_AS` conta espaço de endereçamento, e o YARA-X mapeia o arquivo: com o teto fixo de 1 GiB, um arquivo de 1,1 GiB falhava com `Cannot allocate memory` (medido). O teto passa a ser `DFIR_ENGINE_MAX_MEMORY` + tamanho do arquivo de entrada. O prazo da varredura YARA também cresce com o tamanho (mínimo 60 s; pior caso assumido de 4 MiB/s; teto 6 h), pois 60 s fixos reprovavam o mesmo arquivo. Medido: marcador nos últimos bytes de um arquivo de 1,1 GiB é encontrado.
5. **Supply chain (F1-15, AUD-13).** `cargo deny check` rodado sobre a árvore real (cargo-deny 0.20.2, banco RustSec de 2026-10-11): licenças `BSD-2-Clause`, `ISC` e `BSL-1.0` adicionadas à lista; fontes do egui (`OFL-1.1`, `Ubuntu-font-1.0`) permitidas **somente** para `epaint_default_fonts`; três avisos novos com exceção justificada em `deny.toml` (RUSTSEC-2026-0316 e 0327: wasmtime 45 fixado pelo yara-x, caminho do modelo de componentes não habilitado, verificado com `cargo tree -e features`; RUSTSEC-2026-0192: `ttf-parser` não mantido, só para decoração de janela Wayland); crate retirada `yoke-derive` 0.8.3 atualizada para 0.8.4. Resultado: `advisories ok, bans ok, licenses ok, sources ok`.

## Riscos que permanecem (e por quê)

- **Windows:** a guarda de disco não mede o volume (decisão 2), e nenhuma mudança desta rodada foi compilada ou testada no Windows.
- **Sem verificação no toolchain pinado (1.97.1):** tudo foi compilado e testado em 1.91.1, e a GUI com cópias locais descartáveis de 5 crates de terceiros.
- **Entrada de teclado na GUI:** sob Xvfb o teclado sintético não chegou à caixa de filtro, então o filtro digitado não foi exercitado na GUI (a consulta SQL do filtro tem testes e o filtro roda no ponta a ponta pela CLI). Cliques, rolagem, expansão e a exibição dos avisos foram verificados em captura de tela.
- **Escala:** medido em build de depuração, 1 milhão de eventos indexam em ~49 s (release deve ser várias vezes mais rápido), a página no fim da lista leva ~180 ms (custo de `OFFSET`) e filtro com contagem ~375 ms. Mais que isso (dezenas de milhões) exigiria paginação por chave.
- **EVTX truncado exatamente na fronteira de um chunk, com cabeçalho defasado, pode não gerar aviso** (heurística, não prova).
- **wasmtime 45 do yara-x** segue sem versão corrigida disponível dentro do que o yara-x aceita; reavaliar a cada release do yara-x.
