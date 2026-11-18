<!-- doc-type: decision | status: accepted -->
# ADR-0023 — Tipos `FINDING`, `ROOT_CAUSE` e `PLAN_ITEM`, migration 0010 e plano de conclusão do protótipo

## Status

ACEITA — 2026-10-13, em resposta ao pedido do operador de um planejamento (como, quando e por onde) para finalizar o MVP/protótipo, seguido de "comece pela Etapa 1". Decisão de arquitetura e de ordem de execução (nível 3); **não altera RF**.

## Contexto

1. **Lacuna em relação à demanda.** Dos seis entregáveis da demanda 12412 nenhum é gerado hoje (a única saída é o JSON de IOCs e recomendações). Faltam: Finding e causa-raiz (`RF-051`, F1-25), item de plano (`RF-052`, F1-29 e F1-06), visão NIST por Function (`RF-053`, F1-30) e o gerador de documentos (`RF-013`, F1-07). Ver `docs/DEMAND_TRACE.md`.
2. **Escopo definido pelo operador (2026-10-13).** O projeto é protótipo/MVP, não aplicação de produção; ao avaliar lacunas da demanda, formatos de log além de EVTX ficam fora desta fase.
3. **Restrição técnica.** `analytical_results.result_type` tem `CHECK` com os sete tipos da F1-01 (migration 0006). O SQLite não altera uma constraint existente; aceitar tipos novos exige recriar a tabela. A observação já constava no ticket F1-25 ("o CHECK de `result_type` precisa ser reconstruído"). O comentário da 0006 sobre "não exigir migration a cada novo tipo" refere-se apenas a qual referência (evidência ou incidente) cada tipo exige, validada em Rust — não ao conjunto de tipos.

## Decisão

### D-1 — Plano de conclusão do protótipo (ordem de execução, não corte de escopo)

| Etapa | Ticket(s) | Entrega | Onde (previsto) |
|---|---|---|---|
| 0 | — | Operador roda `just fmt && just check && just test && just nextest && just clippy` e `just e2e <evtx>` na base | — |
| 1 | F1-25, F1-29 (fundação) | Migration 0010 e tipos `FINDING`, `ROOT_CAUSE`, `PLAN_ITEM` | `migrations/0010_*.sql`, `persistence/analytical_results.rs`, `persistence/db.rs` (testes) |
| 2 | F1-25 | Finding e causa-raiz (entregáveis 1 e 2) | `persistence/findings.rs`, `service/findings.rs`, `gui/findings.rs`, `gui/backend.rs`, `gui/incidents.rs` |
| 3 | F1-29, F1-06 | Item de plano; F1-06 vira a visão por domínio, sem código próprio nem Ansible (entregáveis 3, 4 e 5) | `persistence/plan_items.rs`, `service/plan.rs`, `gui/plan.rs` |
| 4 | F1-30 | Visão NIST por Function (GV/ID/PR/DE/RS/RC), descritiva, sem score oficial | `service/nist.rs`, `gui/nist.rs` |
| 5 | F1-07 | Gerador dos cinco documentos Markdown + `manifest.json` com hashes | `service/deliverables/`, `gui/deliverables.rs`, `cli.rs` |
| 6 | F1-31, F1-32 (mínimo) | Proveniência no diálogo de importação, lacunas de retenção e redação de e-mail/usuário na exportação | `gui/evidence.rs`, `service/mod.rs`, `service/deliverables/` |
| 7 | F1-11, F1-12 (reduzidos) | Incidente de demonstração, e2e dos cinco documentos e reconciliação da documentação | `scripts/e2e_headless.py`, `docs/` |

F1-26 (evento normalizado), F1-27 (importador CSV/JSONL) e F1-28 (detecção determinística) **não entram neste plano**: continuam `PLANNED` no backlog e na RTM, e a especificação (§38, Fase 1) segue os listando como MVP. Declarar o MVP completo conforme a especificação exige fechá-los; para o protótipo, o registro manual de vetor, movimentação lateral e exfiltração já existe.

### D-2 — Novos tipos de resultado analítico

| Tipo | RF | Exige incidente | Exige evidência | Exige confiança |
|---|---|---|---|---|
| `FINDING` | RF-051 | sim | não | sim |
| `ROOT_CAUSE` | RF-051 | sim | não | sim |
| `PLAN_ITEM` | RF-052 | sim | não | não |

`ROOT_CAUSE` entra agora, e não na Etapa 2, para não reconstruir a tabela duas vezes. O conteúdo de cada tipo (`payload`) é definido nas Etapas 2 e 3; esta ADR só reserva os tipos. `PLAN_ITEM` não exige confiança porque é uma decisão de plano, não uma inferência sobre evidência.

### D-3 — Migration 0010: recriar a tabela dentro da transação, com `defer_foreign_keys`

O sqlx aplica cada migration numa transação, onde `PRAGMA foreign_keys = OFF` não tem efeito; por isso a receita clássica do SQLite (desligar FKs, recriar, renomear) não serve. A 0010 usa `PRAGMA defer_foreign_keys = ON` (adia as FKs até o COMMIT; o pragma se desliga sozinho): copia as linhas para uma tabela temporária, descarta e recria `analytical_results` com a lista de tipos estendida, reinsere as linhas na mesma ordem de `sequence`, recria os cinco índices e os dois triggers append-only. `analytical_result_relations` não é tocada: referencia a tabela pelo nome. O `DROP TABLE` não dispara triggers, então o append-only não interfere.

Alternativas rejeitadas:

- **`-- no-transaction` com `PRAGMA foreign_keys = OFF`** e a receita de 12 passos: depende de o sqlx honrar a diretiva no SQLite e deixa o banco meio migrado se falhar no meio.
- **Editar `sqlite_master` com `writable_schema`:** exige subir `schema_version` com um valor literal desconhecido na hora de escrever a migration, e as conexões do pool poderiam ficar com o schema antigo em cache, sem aviso.
- **Tabelas próprias para Finding e PlanItem, fora de `analytical_results`:** evitaria a reconstrução, mas perderia `supersedes`, o append-only e as relações `DERIVED_FROM` com recomendações, IOCs e mapeamentos ATT&CK, de que a cadeia Finding → Recomendação → Item de plano depende.
- **Editar a 0006:** o sqlx valida o checksum das migrations aplicadas; bancos existentes passariam a falhar.

### D-4 — Decisões de conteúdo adotadas para as Etapas 2 a 5 (revisáveis)

- Formato dos entregáveis: Markdown + `manifest.json`; PDF fica para uma ferramenta externa, se a entrega exigir.
- A timeline dos documentos traz os eventos citados (achados, estágios do caminho crítico, IOCs) mais total e intervalo; a timeline completa permanece no banco.
- Domínios `POLITICA_IR` e `MONITORAMENTO` entram no Entregável 4 (fortalecimento); `CRIPTO_REPOUSO`, `CRIPTO_TRANSITO` e `GESTAO_CHAVES`, no Entregável 5.
- Seção sem dados é impressa como "sem registro", nunca omitida; cada documento declara que só EVTX é analisado automaticamente.

## Consequências

- Bancos existentes migram sem perda; novas instalações passam pela mesma 0010.
- Qualquer tipo analítico futuro volta a exigir migration com a mesma receita (D-3).
- Risco aceito: a 0010 reescreve uma tabela forense. Mitigação: testes de migração sobre banco populado (ver abaixo). **Lacuna conhecida:** `RF-032` manda copiar e verificar o banco antes de aplicar migration, e o aplicativo não faz isso hoje (nenhum backup em `src/`). Até existir, o operador deve copiar o arquivo do banco (e `evidence/`, se quiser) **antes** de abrir a build com a 0010. Implementar a cópia automática antes de `db::migrate` é pequeno e fica como pendência (RTM: RF-032).

## Verificação (nível exato)

Ambiente: `rustc`/`cargo` **1.91.1** via `apt` (`--ignore-rust-version`: o 1.97.1 pinado não está disponível). Para compilar a GUI, `emath`, `epaint` e `egui` 0.36.2 foram usadas em **cópias locais descartáveis** com 6 substituições equivalentes de APIs do `std` ausentes no 1.91 (`VecDeque::pop_front_if`/`pop_back_if`, `array_windows`, `Vec::push_mut`), fora do projeto e do `Cargo.lock`; isso não valida essas crates no 1.97.1.

- **Etapa 1:** a 0010 aplicada sobre 0001–0009 em SQLite 3.45.1 (Python), com banco populado, vazio e um controle sem o pragma (a migration falha e o rollback preserva as linhas); depois, os testes Rust em rustc 1.91.1: `persistence/db.rs` (migração sobre banco populado e migrator embutido do sqlx) e três testes em `persistence/analytical_results.rs`, todos verdes. O sqlx 0.9 executa o arquivo de vários comandos e o pragma dentro da transação do migrator.
- **Etapa 2:** ver a atualização abaixo.
- Nada é `VERIFIED` no toolchain pinado (1.97.1).

## Atualização — Etapa 2 (2026-10-14): Finding e conclusão de causa (F1-25)

Implementada em `persistence/findings.rs`, `service/findings.rs` e `gui/findings.rs` (aba "Achados"). Não há tabela nova.

- **`FINDING`:** exige título, severidade acima de `NONE` (reuso de `ImpactLevel`, sem `NONE` na interface) e evidência (uma evidência, uma base textual, um mapeamento ATT&CK ou resultados de suporte). Alta confiança (HIGH/CONFIRMED) sem evidência primária exige base textual — a regra de F1-04. Liga-se por `DERIVED_FROM` a mapeamentos ATT&CK do mesmo incidente (validados) e a resultados de suporte; guarda subcategorias NIST validadas só quanto à forma (ADR-0011). `FindingKind`: vulnerabilidade, configuração insegura, controle ausente, credencial fraca, outro.
- **`ROOT_CAUSE`:** três níveis (`ROOT_CAUSE`, `PROBABLE_CAUSE`, `CONTRIBUTING_FACTOR`). O nível causa-raiz só é aceito com confiança HIGH ou CONFIRMED; com menos evidência registra-se causa provável. Toda conclusão cita ao menos um `FINDING` do mesmo incidente. O payload traz `gaps` e `gap_status` (`DECLARED` ou `NONE_DECLARED`): a ausência de lacunas é uma declaração explícita.
- **Revisão:** `supersedes` reaproveita a `correlation_key` e exige o mesmo tipo e o mesmo incidente.
- **Matriz de risco:** `risk_matrix` conta achados por severidade × confiança, só as células existentes e sem pontuação. Atualização `EVD-031`: `findings_view` falha com ID/campo se um Finding vigente tiver severity inválida/`NONE` ou confidence inválida/ausente; não o descarta silenciosamente da contagem nem permite uma matriz zerada nos entregáveis.
- **Serviço:** `create_finding`, `create_root_cause` e `findings_view` (achados, conclusões, mapeamentos ATT&CK e matriz numa só consulta). Campos opcionais em branco valem como ausentes.
- **Limites conhecidos:** a interface não oferece `supporting_result_ids` nem a revisão de um achado; o serviço e a persistência já os aceitam.
- **Verificação:** 154 testes verdes (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos, incluindo 7 testes de persistência, 2 de serviço e 6 da aba (2 de lógica e 4 com `egui_kittest`). Dois testes de interface falharam na primeira execução por defeito do próprio teste (janela baixa demais para alcançar o botão; spinner repintando sob `run`) e foram corrigidos. O clippy também acusou dois `as u64` em `infrastructure/disk.rs`, anteriores a esta etapa (os tipos de `statvfs` variam por plataforma); foi acrescentado `#[allow(clippy::unnecessary_cast)]` com a justificativa.

## Atualização — Etapa 3 (2026-10-14): item de plano (F1-29, absorve F1-06)

Implementada em `persistence/plan_items.rs`, `service/plan.rs` e `gui/plan.rs` (aba "Plano"). Não há tabela nova.

- **Domínios:** os oito do ticket, na ordem das ações da demanda; `domain_coverage` lista **todos**, inclusive os sem itens (a ausência de itens é informação).
- **RB-004:** prioridade (`P1`..`P3`), esforço (`S`/`M`/`L`), classe de custo (`NONE`/`LOW`/`MEDIUM`/`HIGH`) e fase (`IMMEDIATE`/`SHORT_TERM`/`STRATEGIC`) obrigatórios e tipados; sem valores em moeda.
- **RB-005:** `regulatory_reference` é campo próprio, separado de `framework_reference` (CIS, NIST).
- **Recomendação geral:** o vínculo a um `FINDING` do mesmo incidente é opcional; o payload declara `support_kind` (`FINDING_BACKED` ou `GENERAL_RECOMMENDATION`). Isto segue o ticket F1-29 e **substitui** o que o plano da D-1 previa (exigir um achado ou uma recomendação de origem).
- **Obrigatórios além dos acima:** título, motivo e "como verificar". Pré e pós-condição são opcionais.
- **Entregável 5:** os domínios `CRIPTO_REPOUSO`, `CRIPTO_TRANSITO` e `GESTAO_CHAVES` exigem os cinco detalhes (classe do dado, local, mecanismo, rotação, recuperação de chave) e os demais domínios os recusam.
- **Dependências:** `depends_on` aceita itens de plano do mesmo incidente; uma revisão que fecharia um ciclo (direto ou indireto) é recusada.
- **Serviço:** `create_plan_item` e `plan_view` (itens vigentes ordenados por domínio e prioridade, achados vigentes e cobertura numa só consulta). Campos opcionais em branco valem como ausentes.
- **Limite conhecido:** a interface não oferece a revisão de um item; o serviço e a persistência já a aceitam.
- **Verificação:** 171 testes verdes (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos: 7 de persistência, 3 de serviço e 7 da aba (3 de lógica e 4 com `egui_kittest`), incluindo um que impede a divergência entre as listas da interface e o vocabulário do modelo. Ambiente e ressalvas como na atualização da Etapa 2.

## Atualização — Etapa 4 (2026-10-15): visão NIST por Function (F1-30)

Implementada em `persistence/nist_functions.rs` (funções puras), `service/nist.rs` (`function_view`) e na seção "Visão por Function" de `gui/nist.rs`. Sem migration e sem tabela nova.

- **Só contagens.** Por Function (GV, ID, PR, DE, RS, RC, sempre as seis, na ordem oficial): subcategorias do Current por status, subcategorias que o Target elege (exclui `NOT_APPLICABLE`) e gaps vigentes por prioridade. Não há média, percentual nem nota (spec §3: o CSF 2.0 não tem score oficial); a tela traz a nota explícita.
- **Tier fora da tabela (desvio do texto do ticket).** O Tier é do profile inteiro, informado pelo analista (`nist_profiles::Tier`), e não existe por Function. A visão o mostra uma vez para o Current e uma para o Target, com a justificativa. Derivar um Tier por Function seria inventar um valor.
- **Gap desatualizado.** `gap_analysis_state` compara os ids de profile gravados na proveniência de cada gap vigente com os profiles vigentes: `UP_TO_DATE`, `STALE`, `MISSING_PROFILE` ou `NO_GAPS_RECORDED`. A rodada de gaps grava um resultado novo para todo gap, então depois de recalcular todos carregam os ids vigentes. `NO_GAPS_RECORDED` não distingue "análise não rodada" de "Current já atinge o Target", porque rodar sem gaps não grava nada; a tela diz as duas coisas.
- **Atualização da tela:** a visão é pedida ao abrir a aba e depois de criar um profile e de recalcular os gaps.
- **Testes de serviço:** fixture comum em `service/test_support.rs`, usado agora também pelos testes de achados e de plano (refatoração só de testes).
- **Verificação:** 180 testes verdes (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos: 4 de agregação pura, 1 de serviço (sem profiles, com profiles sem gap, gaps em dia, gaps desatualizados após nova versão do Current e recalculo) e 4 de interface com `egui_kittest`. Ambiente e ressalvas como nas atualizações anteriores.

## Atualização — Etapa 5 (2026-10-16): gerador dos entregáveis (F1-07)

Implementada em `service/deliverables/` (`markdown.rs`, `mod.rs` e um renderizador por documento), na aba "Entregáveis" (`gui/deliverables.rs`) e no subcomando `dfir-app deliverables`. Sem migration.

- **Cinco documentos Markdown e um `manifest.json`**, gravados em `<pasta>/entregaveis-<incidente>/`: `01-relatorio-forense.md`, `02-vulnerabilidades-attack.md`, `03-nist-recomendacoes.md`, `04-hardening.md`, `05-criptografia.md`. Os domínios `POLITICA_IR` e `MONITORAMENTO` ficam no Entregável 4; criptografia em repouso e em trânsito e gestão de chaves, no 5 (decisão D-4).
- **Nada é inventado.** Cada documento só reorganiza dados registrados: item cita o resultado de origem (id curto no texto; ids completos por documento no manifest) e seção sem dados diz "sem registro", em vez de sumir. Não há pontuação nem média em lugar nenhum.
- **Escopo declarado.** O Entregável 1 afirma que a análise automática é só de logs EVTX; o que consta de outras fontes é registro manual. Avisos de análise, falha na cadeia de custódia, mapeamentos e estágios sem base documentada e lacunas das conclusões de causa entram em "Escopo e limitações".
- **Matriz de proteção (Entregável 5):** classe do dado × (repouso, trânsito, chaves), montada dos cinco detalhes dos itens de plano. Uma célula sem item aparece como "— (sem item)": é uma lacuna do plano.
- **Timeline nos documentos:** os marcos do caminho crítico e, por evidência, total de eventos, primeiro, último e avisos. A linha do tempo completa permanece no banco (decisão D-4).
- **Determinístico:** a renderização é uma função pura de `DeliverableData`; as listas têm ordenação explícita com o id como desempate, e um teste confirma saída idêntica com a entrada em outra ordem. O único dado que muda entre execuções é `generated_at`, só no manifest. Documento vazio ou sem título falha a geração.
- **Seguro:** todo texto de dados passa por `esc` (escapa o que o Markdown interpreta e colapsa quebras de linha e caracteres de controle, inclusive `ESC`) ou `code` (trocadas as crases; `|` protegido). Os nomes de arquivo são constantes. A pasta de saída recusa o armazenamento interno da aplicação (evidências, banco, quarentena, logs...), links simbólicos e caminho ocupado por arquivo; o id do incidente, que entra no nome da pasta, só aceita caracteres seguros. A escrita é por arquivo temporário + renomeação.
- **Auditoria:** o evento `DELIVERABLES_GENERATED` guarda só nomes de arquivo e hashes (sem caminhos).
- **Manifest:** esquema `dfir-deliverables/1`, com incidente, `generated_at`, versão da ferramenta, o escopo, e por documento: nome, título, bytes, SHA-256, BLAKE3 e ids dos resultados citados.
- **Fora desta etapa (Etapa 6):** opção de redação (F1-32) e lacunas de proveniência e retenção (F1-31). Por isso F1-07 e DEM-ENT-01 ficam parciais.
- **Limite conhecido:** o botão "Gerar arquivos…" abre o seletor nativo de pasta, que os testes de interface não acionam; a geração é exercida pelo serviço, pela CLI e pelo comando da aba.
- **Verificação:** 200 testes verdes (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos: 5 do construtor de Markdown, 8 do gerador (renderização, texto hostil, determinismo e ordem, escrita atômica e hashes, recusas de pasta, geração com auditoria), 6 da aba e 1 da CLI. Ajustes durante a verificação: a tabela de auditoria chama-se `audit_events`; o teste de injeção precisou distinguir trechos de código (literais pelas regras do Markdown) do restante do texto; e **ler os documentos renderizados revelou um defeito que as asserções não pegavam**: "Ativos afetados" e "Dados afetados" saíam vazios no Entregável 1 porque o payload de impacto traz lista e o código lia só texto. Corrigido (`field_text`) e coberto por asserção. Também trocados os cabeçalhos `#` (que o escape deixava como `\#`) por `Nº` e o cabeçalho `Severidade \\ Confiança` por `Severidade × Confiança`.

## Atualização — Etapa 6 (2026-10-18): proveniência, lacunas de fonte e máscara (F1-31 e F1-32)

Implementada em `migrations/0011_provenance_and_source_gaps.sql`, `persistence/source_gaps.rs`, `service/provenance.rs`, `service/gaps.rs`, `service/deliverables/redaction.rs`, na aba "Lacunas" (`gui/gaps.rs`), no formulário e na coluna "Tratamento" de `gui/evidence.rs`, na opção da aba "Entregáveis" e nas flags da CLI.

- **Migration 0011** reutiliza a receita da D-3: recria `analytical_results` com `SOURCE_GAP` (dentro da transação, com `defer_foreign_keys`) e acrescenta duas colunas a `evidence`: `provided_by` e `contains_personal_data` (0 ou 1). `evidence` não tem trigger de imutabilidade, então o `ALTER TABLE` é direto. Verificada em SQLite 3.45.1 e pelo migrator embutido do sqlx.
- **Proveniência declarada, nada suposto.** A importação gravava constantes (`LOCAL_OPERATOR`, `LOCAL_FILE_PICKER`) e **a data de importação no lugar da data de coleta**. Agora grava o que o operador declara (origem, quem forneceu, método, ferramenta e versão, data da coleta, autorização, flag de dado pessoal) e `NOT_PROVIDED` ou ausente no resto. Linhas antigas continuam como estão, mas `declared_source`, `declared_collection_method` e `declared_collected_at` as tratam como não informadas (a constante de importação e a data de importação igual à de coleta não contam como declaradas). A validação (data válida, uma linha, 500 caracteres) acontece **antes** de qualquer cópia de arquivo.
- **Tratamento declarado:** só EVTX é analisado automaticamente; o resto é "somente preservado". A aba de evidências mostra isso por evidência, e o Entregável 1 também.
- **Lacunas de fonte (RB-001):** `SOURCE_GAP` guarda fonte esperada (obrigatória), período opcional (data ou RFC 3339, início não depois do fim), motivo (retenção, não fornecido, não coletado, outro; `OTHER` exige nota) e nota. Entra no Entregável 1 (seção 2.1), no resumo e em "Escopo e limitações", ao lado da proveniência não informada de cada evidência. A ausência de lacunas registradas é dita como tal e **não** como prova de que não houve.
- **Máscara de dados pessoais (RB-006), ligada por padrão.** Reconhece IPv4, IPv6, e-mails e contas `DOMÍNIO\usuário` por padrão de texto (com guardas contra falsos positivos: timestamps, versões, hashes, UUIDs, caminhos como `C:\Windows\System32`), e mascara por inteiro o campo "fornecida por" e o responsável de cada evento de custódia (exceto o ator padrão do aplicativo). **Não detecta nomes de pessoas em texto livre**; o documento e a tela dizem isso.
  - **Marcadores estáveis** (`{IP-1}`, `{EMAIL-2}`, `{CONTA-1}`, `{PESSOA-3}`): o mesmo valor recebe o mesmo marcador em todos os documentos; o mapeamento não é gravado. A numeração segue a ordem alfabética dos valores e vem de dois passes (coleta, depois substituição), então a saída não depende da ordem dos dados.
  - **Exportar sem máscara é escolha explícita:** a interface mostra o aviso (e quantas evidências foram marcadas com dado pessoal) e só libera a geração após uma confirmação; a CLI exige `--no-redaction`. O manifest e a auditoria registram se foi mascarado e o quê, **só por contagem**.
- **Ordem de entrada e determinismo:** o teste de ordem revelou que a lista de evidências do Entregável 1 seguia a ordem recebida; `render_all` agora normaliza a ordem das evidências e dos eventos de custódia antes de renderizar.
- **Compromisso conhecido:** com a máscara ligada, os IPs de IOC também são mascarados, e um IOC de IP é justamente o que o cliente precisa para bloquear. É o padrão protetor; para entregar a quem vai agir sobre os indicadores, o operador desliga a máscara conscientemente.
- **Limite:** a interface não oferece a revisão de uma lacuna (o serviço e a persistência aceitam).
- **Verificação:** 230 testes verdes (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos. Novos: migration 0011 sobre banco populado e pelo migrator embutido; lacunas (persistência e serviço); proveniência (importação com e sem declaração, validação antes da cópia, linhas antigas); redação (detecção e falsos positivos, marcadores estáveis, nenhum valor original nos cinco documentos nem no manifest, determinismo e ordem de entrada, auditoria da escolha); formulário e resumo de proveniência, aba de lacunas e opção de máscara na interface; flags da CLI. Ajustes durante a verificação: três erros de lifetime em `gui/gaps.rs` (closure e `map_or` com `&'static str`), um teste com numeração de IP errada (a ordem é alfabética), `json_extract` devolve inteiro no SQLite, a coluna da auditoria chama-se `object_id`, e o código morto que o clippy apontou (um wrapper e um método sem uso).

## Atualização — Etapa 7 (2026-10-23): incidente de demonstração, e2e dos cinco documentos e reconciliação (F1-11 e F1-12, reduzidos)

Implementada em `service/demo.rs`, no subcomando `dfir-app demo`, em `scripts/e2e_headless.py`, em `docs/DEMO.md` e `docs/OPERATION.md`, e na reconciliação dos documentos vivos. Sem migration.

- **O cenário vive no binário e entra pelo serviço.** `service/demo.rs` registra o cenário BEC da spec §42 com as mesmas funções que a interface chama (nenhuma regra é contornada: o conteúdo precisou passar pelas validações reais). Alternativas rejeitadas: um arquivo JSON de seed (exigiria referências simbólicas entre registros e um interpretador, com menos conferência na compilação) e o script Python gravando no banco (`AGENTS` §2: o script não toca o banco). Custo aceito: o texto do cenário fica em Rust.
- **Real e sintético, sem mistura.** O `.evtx` informado é importado e analisado de verdade (timeline e YARA-X). Tudo o que um analista registraria é sintético, **não decorre do `.evtx`** e diz isso em cada base. IOCs só em faixas de documentação (RFC 5737, `.example`); a pessoa citada é fictícia. O `.evtx` **não é distribuído** com o projeto; origem e hash em `tests/fixtures/README.md`.
- **Rótulo DEMO (spec §21.3) em todo lugar útil:** classe da evidência (`status = DEMO`, nova função `import_evidence_classified`; a interface e a CLI de importação seguem `FORENSIC`), nome do incidente (título dos cinco documentos), proveniência, base de cada registro e auditoria (`DEMO_DATASET_SEEDED`, com versão `bec-demo-1` e contagens).
- **Roteiro e operação:** `docs/DEMO.md` (roteiro offline, o que é real e o que é sintético) e `docs/OPERATION.md` (instalação de referência, proteção do volume — RF-030 —, backup e atualização/rollback manual — parte documental de RF-032).
- **e2e dos cinco documentos:** de 38 para 79 verificações em `scripts/e2e_headless.py`, com engine e EVTX reais: contagens do cenário, jobs reais concluídos, hashes do JSON e do `manifest.json` contra os arquivos, rótulo DEMO nos cinco, ausência de seção vazia e de HTML bruto, tabelas bem formadas, os três grupos de cada documento, a lacuna da matriz, máscara (valores somem, marcadores aparecem), geração sem máscara, determinismo byte a byte entre duas gerações, descarte da evidência e `audit-verify` antes e depois.
- **Três defeitos reais, achados por rodar o caminho inteiro:**
  1. **`analyze` falhava em toda evidência** desde a Etapa 6 (`Database(ColumnNotFound("source"))`): `analyze_inner` lia só 9 colunas de `EvidenceRow`, que passou a exigir `source` e `collection_method`. A consulta é erro de **execução**, não de compilação, e nenhum dos 230 testes a exercia; o e2e já existente a pegou (15 de 22). Corrigido com uma lista única de colunas (`evidence_columns!`) nas duas consultas e coberto por teste que falha com a consulta antiga.
  2. **Evidência descartada sumia dos documentos:** depois do descarte local o Entregável 1 dizia "nenhuma evidência citada", embora IOCs a citem. Agora a evidência descartada continua listada (hash e custódia com `DESTROYED`), com a nota de que o arquivo foi descartado e a análise removida. Coberto por teste.
  3. **Limite descoberto no Entregável 5 (não é defeito do código):** a matriz de proteção agrupa pelo **texto exato** da classe do dado. Itens do mesmo dado escritos de formas diferentes viram linhas separadas, com lacunas aparentes. O cenário usa o mesmo texto; o operador precisa fazer o mesmo.
- **F1-11 `IMPLEMENTED / PENDING-HUMAN` (reduzido):** dataset BEC reproduzível (sintético, só EVTX) com proveniência e roteiro documentado. **Fora:** CSV/JSONL (F1-27 não existe) e, para os testes de injection/traversal, só valem os que já cobrem os documentos (Etapa 5: texto hostil, pasta de saída, ids) e a proveniência (Etapa 6); não há testes de envelope JSONL nem de mapeamento CSV porque não há importador.
- **F1-12 `PARCIAL`:** reconciliação feita (README, CHANGELOG, backlog, RTM, DEMAND_TRACE, RELEASE_GATE, prompt de continuidade); documento de operação com RF-030 e a parte documental de RF-032. **Não cumpre** o critério "nenhuma linha da RTM em `PENDING`/`PLANNED` para RFs MVP": F1-26, F1-27 e F1-28 seguem `PLANNED`; a cópia automática do banco antes de migration (RF-032) segue pendente; nada está `VERIFIED` no toolchain pinado.
- **Verificação:** 240 testes verdes (1 ignorado) em `cargo test -p dfir-app`; `cargo clippy -p dfir-app --all-targets --all-features -- -D warnings` e `rustfmt --check` limpos; `python3 scripts/e2e_headless.py --evtx ...` **79/79** com engine e `security.evtx` reais (binários de depuração já compilados; a receita `just e2e` não foi executada) (SHA-256 `5f29a03c…9a4d8d`). Ambiente: rustc/cargo 1.91.1 (apt) com cópias locais descartáveis de `emath`/`epaint`/`egui` e, para o engine, de `wasmtime-environ`/`wasmtime` (substituições equivalentes de APIs instáveis do `std` no 1.91, fora do projeto e do `Cargo.lock`). Novos testes: 7 do seed (cenário completo, classe DEMO e auditoria, rótulo nos cinco documentos, domínios e lacuna da matriz, máscara, determinismo, classe desconhecida recusada), 1 do descarte, 1 da regressão do `analyze`, 1 do parsing de `demo`.
- **Não verificado:** toolchain 1.97.1 pinado; a GUI numa janela real; a execução offline de fato (o ambiente tinha rede, usada só para obter dependências e o `.evtx`; o aplicativo em si não usa rede, mas isso não foi medido com a rede cortada); `cargo test` do engine e do workspace (só o crate do aplicativo foi testado; o clippy do workspace com 1.91 acusa 3 `assert!(true)` antigos nos testes de `protocol`, `assertions_on_constants`, não avaliados no 1.97.1); a geração com um `.evtx` real que contenha um ataque (o cenário é sintético por definição).
- **Próximos passos recomendados:** cópia automática do banco antes de migration (RF-032), pequena e de risco real sobre dados reais; verificação no toolchain pinado pelo operador (F1-14); F1-26..F1-28 para fechar o MVP conforme a especificação.
