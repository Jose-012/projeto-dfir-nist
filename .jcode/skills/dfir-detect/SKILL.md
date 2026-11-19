---
name: dfir-detect
description: Guia inclusão e alteração de regras determinísticas EVTX no DFIR/NIST, com campos normalizados, candidatos confirmáveis, testes de fronteira e documentação sincronizada
allowed-tools: agentgrep, read, edit, apply_patch, write, bash, todo
---

# Alteração controlada de regras de detecção

Use esta skill ao incluir, alterar ou remover regras internas de `EVTX_DETECTION` em `engine/src/detect.rs`, inclusive ao ajustar normalização indispensável, testes ou catálogo dessas regras.

Não use esta skill para:

- alterar o ruleset YARA-X, que tem contrato e gates próprios;
- adicionar importadores ou declarar suporte a fontes fora do escopo EVTX;
- confirmar ou rejeitar candidatos de um caso, pois essa decisão pertence ao analista;
- transformar uma técnica de ataque interessante em requisito sem rastreabilidade e autoridade.

## 1. Autoridade e escopo

1. Leia `AGENTS.md`, `docs/adrs/ADR-0025-eventos-normalizados-deteccao-por-regras-e-escopo-evtx.md` e `docs/DETECTION_RULES.md`. Em conflito documental, consulte `docs/DOCUMENT_AUTHORITY.md`.
2. Identifique o item da demanda e o requisito cobertos. Não transforme uma hipótese, um exemplo de ataque ou uma heurística útil em requisito. Se a mudança alterar escopo, protocolo ou decisão de arquitetura, siga o processo de autoridade e ADR em vez de escondê-la em uma regra.
3. Mantenha o escopo atual em EVTX. A importação CSV/JSONL e fontes de rede, SIEM, dispositivos de segurança e e-mail/identidade na nuvem estão adiadas pela ADR-0025; não declare suporte a elas.
4. Rode `git status --short` e preserve mudanças alheias. Faça a menor mudança suficiente, sem introduzir dependência ou runtime novo para uma regra.

Uma regra produz somente **candidato pendente**. Ela não cria achado, mapeamento ATT&CK ou IOC confirmado: isso exige decisão do analista. Ausência de candidato não prova ausência de ataque. Confiança automática é apenas `LOW` ou `MEDIUM`, e `limitations` deve declarar o que a regra não prova, incluindo falsos positivos plausíveis e auditorias/fontes necessárias.

## 2. Entradas mínimas

Antes de editar, reúna ou delimite explicitamente:

- requisito, item da demanda ou ticket e a ADR aplicável;
- Event IDs, provedores, campos EVTX brutos e auditorias do Windows necessários ao sinal;
- campos normalizados consumidos e comportamento esperado quando cada um estiver ausente;
- ID e versão da regra, metadados ATT&CK, gravidade, confiança e limitações do método;
- predicado, limiar/janela, agrupamento, período, referências e IOCs esperados;
- fixtures sanitizadas ou eventos sintéticos positivos, negativos próximos e de fronteira;
- documentos, testes e gates afetados, separando mudança local de regra de mudança de contrato.

Se um desses dados for desconhecido, registre `ASSUMPTION` ou `PENDING-HUMAN` conforme o caso e não o apresente como fato. Não leia dados de caso quando uma fixture sanitizada bastar.

## 3. Campos normalizados

- Consuma `Ev.fields` (`EventFields`) e `ev.id`; os campos estruturais são opcionais: `event_id`, `provider`, `channel`, `host`, `user`, `src_ip`, `dst_ip`, `process`, `command_line`, `action`, `outcome` e `source_type`. Campo necessário ausente deve impedir o disparo, não ser adivinhado.
- Vazio, `-` e `(NULL)` representam ausência. A normalização remove NUL e limita texto comum a 512 caracteres e comando/bloco de script a 4096. Não contorne esses limites.
- Para dados específicos do evento, use `Ev::d` e os extratores de `dfir_protocol::normalize`. Engine e app compartilham `normalize::event_fields`; não crie no detector uma interpretação divergente da timeline.
- Se precisar mudar a extração, altere `protocol/src/normalize.rs` e seus testes, preservando as formas EVTX aceitas e a compatibilidade dos resultados persistidos. Se precisar de um campo ou contrato novo, trate a mudança de esquema/app/engine explicitamente, não como edição local de regra.
- Casamento por texto só vê a tentativa registrada e pode falhar com renomeação/ofuscação. Sem auditoria do evento correspondente não há sinal; sem log de rede não se confirma exfiltração nem volume transferido.

## 4. Implementação

1. Defina um `Rule` com `id`, `version`, `title`, `description`, `tactic`, `technique`, `sub_technique`, `severity`, `confidence` e `limitations`. Use ID único `R-<CATEGORIA>-NNN`, sem reutilizar IDs. Tática deve ser uma das 14 aceitas em maiúsculas; técnica e subtécnica devem manter a relação validada pelo catálogo. A validação de forma não prova que o mapeamento ATT&CK está semanticamente correto.
2. Inclua a regra em `catalogue()` e ligue seu predicado no dispatch de `Detector::observe` ou no caminho já existente de comando/rede. Teste provedor, Event ID, ação, resultado e campos obrigatórios conforme o sinal; não dispare por Event ID isolado quando ele for compartilhado ou ambíguo.
3. Reuse `Detector::agg` e `Agg`. Escolha agrupamento determinístico e suficiente para distinguir candidatos. A chave estável combina regra, versão e agrupamento; reexecução não pode duplicar nem desfazer decisões. Ao mudar agrupamento ou comportamento, verifique explicitamente o efeito sobre essa identidade e sobre candidatos já decididos.
4. Use `agg.add(ev)` para referências `record:N` e registre somente IOCs realmente observados, com sua referência. Preserve os tetos de 5 000 candidatos, 50 000 chaves de rajada, 20 referências e 20 IOCs por candidato, a marca de truncamento e o aviso `DETECTION_LIMIT_REACHED`. Não crie estado sem teto nem oculte overflow.
5. Ao criar ou remover uma regra, ou alterar seu limiar, mude `RULE_SET_VERSION`. Não altere versões ou chaves mecanicamente: relacione qualquer mudança à rastreabilidade e à idempotência das decisões.

## 5. Testes obrigatórios da regra

Use os helpers sanitizados `record`, `security` e `Feed` em `engine/src/detect.rs` sempre que adequados. Inclua casos com asserções de presença **e** ausência:

- **positivo:** o sinal exato gera o candidato esperado; verifique regra/versão, metadados, agrupamento, contagem, referências e IOCs pertinentes;
- **negativo:** evento vizinho, provedor incorreto, variante legítima, campo obrigatório ausente e texto apenas parecido não disparam;
- **fronteira:** para limiar temporal ou numérico, teste imediatamente abaixo, no limite e fora da janela; para texto, teste delimitadores e substrings que não devem casar;
- **robustez:** placeholders, tipos JSON aceitos/ausentes, valores longos/NUL e timestamp inválido não produzem inferência nem pânico;
- **determinismo:** mesma entrada preserva `candidate_key` e `RULE_SET_VERSION`; agrupamentos diferentes não colidem.

Se a normalização mudar, adicione casos em `protocol/src/normalize.rs` para `EventData`/`UserData`, formas de Event ID e campos ausentes, conforme o caminho alterado. Se serviço ou persistência mudar, cubra também confirmação, rejeição motivada, finalidade e reexecução idempotente; uma regra isolada não deve reescrever essa camada.

Quando um casamento por texto aceitar um falso positivo conhecido, não o disfarce como teste negativo nem como detecção precisa. Preserve o caso que demonstra o limite e descreva o falso positivo em `limitations`, como já ocorre com texto contendo nome de ferramenta em `R-CRED-001`.

## 6. Limites da confirmação

- O app inicia candidatos em `PENDING`. Somente a confirmação do analista cria, no incidente escolhido, mapeamento ATT&CK, achado e IOCs, mantendo regra, versão e eventos de origem na base.
- Rejeitar exige motivo. Confirmação e rejeição são finais: não reabra a decisão nem altere o que a regra afirmou para acomodar uma edição posterior.
- Reexecutar a detecção não pode duplicar candidatos nem desfazer decisões. Se mudar regra, versão ou agrupamento, teste o efeito na chave estável antes de afirmar idempotência.
- Não introduza confirmação em lote nem automática no caminho de uma regra. O protótipo decide um candidato por vez.
- Pendente não é achado. Relatórios distinguem confirmados, rejeitados e pendentes, além de "regras não executadas" e "evidência descartada"; não confunda "nada disparou" com "não rodou".
- Falha na confirmação deve permanecer diagnosticável e a retomada não pode duplicar os artefatos já gravados. Testes do detector não provam esse fluxo de serviço e persistência.

## 7. Documentação gerada

1. Gere o catálogo real com:

   ```bash
   cargo run --locked -p dfir-engine -- --list-rules
   ```

2. Regenere `docs/DETECTION_RULES.md` a partir dessa saída e do `RULE_SET_VERSION`: tabela de regras, seção "O que cada regra não prova", cobertura e limiares afetados. A saída é JSON, não o Markdown pronto; não a redirecione por cima do documento. Não edite apenas o cabeçalho para satisfazer o teste.
3. O teste de documentação confere versão do conjunto, IDs/versões e `limitations`; ele não compara toda descrição, gravidade, tática ou limiar. Revise esses campos no diff.
4. Atualize somente demais documentos afetados e a rastreabilidade exigida pela autoridade. Não crie ADR, evidência `PASS` ou narrativa permanente de rotina sem necessidade e execução correspondente.

## 8. Gates e fechamento

Execute os gates seletivos primeiro e depois os gates completos aplicáveis:

```bash
cargo fmt --all -- --check
cargo test --locked -p dfir-engine detect::tests
# Se a normalização mudou:
cargo test --locked -p dfir-protocol normalize::tests
# Se serviço, persistência ou confirmação mudou:
cargo test --locked -p dfir-app service::detection::tests
just verify
just doc-gate
```

Quando o fluxo com engine real for afetado, use `just e2e tests/fixtures/senai-demo.evtx` ou outra fixture sanitizada adequada. Confirme se a fixture realmente exercita a regra; zero candidatos ou um pipeline verde sem o sinal esperado não prova a detecção. Rode `just supply-chain` somente se o grafo de dependências/advisories for afetado, sem dispensar outros gates por isso.

Termine com `git diff --check`, revisão semântica do diff e `git status --short`. Relate comandos, resultados e pendências exatamente; não converta teste seletivo, `WAIVED`, fixture DEMO ou inspeção de código em `VERIFIED` do produto ou de incidente real.
