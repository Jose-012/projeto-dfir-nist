<!-- doc-type: decision | status: accepted -->
# ADR-0016 — Engine em módulos e `NormalizedEvent` tipado no `dfir-protocol`

## Status

ACEITA — 2026-09-21

## Contexto

Duas fragilidades do engine depois de F1-02:

1. `engine/src/main.rs` reunia entrada, limites, saída e o roteamento por `analysis_kind` (`process_request` com dois braços, EVTX e YARA). A spec já aponta PCAP (seção 11) e ClamAV (seção 12) como próximos formatos: o `match` só cresce.
2. O evento de timeline era um `json!({...})` montado à mão no engine (`normalized_item`), e o app (`timeline_event_from_item`) **adivinhava** essa forma por duck-typing (`.get("campo")`). Um acoplamento invisível: renomear um campo no engine só quebraria o app em tempo de execução, em silêncio.

## Decisão

### `NormalizedEvent` no `dfir-protocol`

O evento de timeline vira o struct `dfir_protocol::NormalizedEvent` (`event_record_id`, `timestamp`, `data`, `evidence_id`, `raw_reference`, `normalization_status`), com `into_item()` (engine) e `from_item()` (app). Mora no protocolo porque é o formato fundacional da Fase 0 que sustenta a ferramenta — diferente do item de match YARA, mantido genérico de propósito na ADR-0010 (é um "tipo de análise entre vários"). `RESULT.items` continua `Vec<Value>`: o protocolo não vira uma enum de todos os tipos de item.

- O JSON no fio **não mudou** (mesmos nomes e tipos de campo); só `normalization_status` ganhou default ao ler.
- `from_item` devolve `None` para itens que não seguem o contrato. O app agora é **mais estrito** que o duck-typing: um item sem `evidence_id`, `raw_reference` ou `data` é ignorado, em vez de cair em valores como `"unknown-evidence"`. O engine sempre emite esses campos.

### Engine em módulos

`engine/src/main.rs` ficou só com o ponto de entrada; o resto foi dividido, sem mudar comportamento:

| Módulo | Responsabilidade |
|---|---|
| `runner` | `EngineRunner`: recebe um `Request` já validado, resolve a entrada e roteia por `analysis_kind` para `process_evtx_request` ou `process_yara_request`; `analysis_kind` desconhecido vira `UNSUPPORTED_ANALYSIS_KIND`. É onde PCAP/ClamAV entram como novos braços. |
| `input` | Raiz de entrada (`input_root`), resolução e validação do caminho (`resolve_input`) e `reject_symlink`. |
| `limits` | Limites internos do engine: `MAX_RECORD_ERRORS` (64) e `RESULT_BATCH_SIZE` (128). Os limites de linha e profundidade JSON continuam em `dfir-protocol`, porque app e engine precisam aplicar os mesmos valores. |
| `output` | Emissão de linhas do protocolo: `emit_protocol_line`, `emit_result_batch`, `result_line_size`. |
| `evtx_timeline` | Funções de apoio do EVTX: extensão do arquivo, extração e normalização do timestamp do evento. |
| `error` | `EngineFailure`, o par `(código estável, mensagem)` que vira um `ErrorMessage` do protocolo. |
| `yara_scan` | Já existia (F1-02). |

`reject_symlink` é a mitigação de TOCTOU já testada. A conversa de planejamento condicionou mexer nele a "ter um primeiro `cargo build` limpo"; essa condição foi cumprida antes da mudança (build e testes verdes sobre o código original, incluindo a correção de erros latentes de compilação do app), e o código foi **movido**, sem alteração de lógica, com seus testes.

### O que **não** foi feito, de propósito: trait `Parser` / `EvtxParserAdapter` / `ParsedRecord`

Existe hoje **um** parser iterativo (EVTX). YARA não conta como segundo caso do padrão: ele varre o arquivo inteiro, não itera registro a registro. Extrair uma trait a partir de uma única implementação é adivinhar a interface pensando em "como seria o PCAP", sem o PCAP na mão para validar a forma — e o `CONVENTIONS.md` do projeto pede para evitar essa generalização especulativa. **Gatilho para retomar**: quando PCAP (ou outro formato iterativo) virar tarefa real, desenhar a trait junto com ele, com dois casos concretos guiando a interface.

## Consequências

Positivas: contrato do evento verificado pelo compilador nos dois lados; o crescimento por novos formatos tem um lugar óbvio (`runner`); arquivos pequenos e testáveis por módulo.

Negativas: o app rejeita mais estritamente itens malformados (ver acima); mais arquivos para navegar.

## Evidência de execução

`cargo test -p dfir-engine` (30 testes, incluindo os movidos), `cargo test -p dfir-protocol` (10) e `cargo test -p dfir-app` verdes; `reject_symlink` foi testado apenas em Linux — rodar `cargo test -p dfir-engine` no Windows antes de considerar a mudança fechada nessa plataforma.
