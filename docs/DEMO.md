<!-- doc-type: reference | status: current -->
# Roteiro da demonstração offline (F1-11, ADR-0023)

Mostra o caminho completo — evidência real, análise real, cenário registrado, cinco documentos — em poucos comandos. O que é real e o que é sintético está dito abaixo; **nada aqui é uma investigação de verdade**.

## O que é real e o que é sintético

| Parte | Origem |
|---|---|
| O arquivo `.evtx`: hash, custódia, proveniência declarada | **Real.** É o arquivo que você informa; é importado e preservado como qualquer evidência |
| Linha do tempo, varredura YARA-X e **detecção por regras** | **Real.** O engine analisa o arquivo de verdade, contido pelo sandbox no Linux. A detecção gera **candidatos pendentes** a partir do que o arquivo realmente contém (no `security.evtx` de amostra: 2 contas criadas, as duas adicionadas a Administrators e 2 contas com privilégios sensíveis); **nada é confirmado automaticamente**, e o cenário BEC abaixo continua sintético e independente deles |
| ATT&CK, caminho crítico, impacto, IOCs, achados, causas, recomendações, plano, lacunas de fonte, profiles NIST | **Sintético.** Escritos em `src/service/demo.rs` (cenário BEC da spec §42) e registrados pelas mesmas funções de serviço que a interface usa |

O cenário **não decorre do `.evtx`**: a amostra pública é um log comum, sem ataque, e cada texto sintético diz isso. IPs, domínios e e-mails são de faixas reservadas para documentação (RFC 5737, TLD `.example`); o hash do IOC `SHA256` é de um texto fixo, sem arquivo correspondente; a pessoa citada (`Ana Exemplo`) é fictícia.

O rótulo **DEMO** (spec §21.3) está na classe da evidência (`status = DEMO`), no nome do incidente (`[DEMO] ...`, que vira o título dos cinco documentos), na proveniência, na base de cada registro e na auditoria (`DEMO_DATASET_SEEDED`). Versão do conteúdo: `bec-demo-1`.

## Pré-requisitos

- Build: `cargo build --locked -p dfir-app -p dfir-engine` (o engine precisa estar ao lado do app ou em `DFIR_ENGINE_PATH`). O `-p` é necessário porque a raiz é package e workspace ao mesmo tempo; `--bin dfir-engine` sozinho aponta para o pacote errado.
- Um arquivo `.evtx`. O projeto **não distribui** nenhum. O usado nas verificações é `security.evtx`, do repositório `omerbenamram/evtx` (pasta `samples/`): 2 166 784 bytes, SHA-256 `5f29a03cf8c1b4bfbd82c4074ccc81ecfe85d7ee15375afc352ed5025c9a4d8d`. O repositório é licenciado sob MIT ou Apache-2.0; **a procedência do arquivo em si não é documentada lá** (não verificada). Qualquer outro `.evtx` serve: o cenário não depende do conteúdo.

## Passo a passo

```bash
export DFIR_DATA_ROOT=/caminho/para/uma/pasta/nova        # recomendado: dados da demo separados dos reais
dfir-app demo caminho/security.evtx                        # JSON: incident_id, evidence_id, jobs e contagens
dfir-app deliverables <incident_id> --out ./saida          # 5 documentos + SVG + manifest.json, dados pessoais mascarados
dfir-app deliverables <incident_id> --out ./saida-bruta --no-redaction   # os mesmos, sem máscara (compare)
dfir-app deliverables <incident_id> --out ./saida-assinada --sign-with /caminho/externo/chave.hex
dfir-app verify-deliverables ./saida-assinada --public-key <chave_publica_hex>
dfir-app audit-verify                                      # cadeias de auditoria e de custódia
dfir-app                                                   # a interface: o incidente [DEMO] já está na lista
```

Ou, com `just`: `just demo caminho/security.evtx`. Cada execução de `demo` cria um **incidente novo** e importa a evidência de novo; use uma pasta de dados nova para começar limpo.

Para ver a revisão de candidatos de detecção (o que a demo deixa pendente): `dfir-app candidates <evidence_id>` lista as hipóteses com regra, ATT&CK sugerida e eventos de origem; `candidate-confirm`/`candidate-reject` decidem (ver `docs/OPERATION.md` §3C). Os candidatos da amostra **não** pertencem ao cenário BEC; confirmar um deles o acrescenta ao incidente `[DEMO]` como um achado real do que o arquivo contém.

Na interface, abra o incidente `[DEMO]` e percorra as abas (achados, plano, NIST, lacunas, entregáveis). A aba "Entregáveis" mostra a pré-visualização e gera os arquivos.

## Onde a cadeia do cenário aparece

| Etapa do cenário (spec §42) | Registro | Documento |
|---|---|---|
| E-mail de phishing | ATT&CK T1566.002; estágio 1 do caminho; IOCs URL, domínio e e-mail | 1, 2 |
| Credencial capturada, login anômalo | T1078.004; achado sem segundo fator; IOC de IP | 1, 2 |
| Regras de caixa (encaminhamento, ocultação) | T1114.003, T1564.008; achado de encaminhamento | 1, 2, 4 |
| PowerShell, movimento lateral, escalonamento | T1059.001, T1021.001, T1098; achados de RDP e de PowerShell | 2, 4 |
| Exfiltração e fraude de pagamento | T1567, T1657; impacto | 1, 2 |
| Causas | uma raiz, uma provável (com lacunas), um fator contribuinte | 1, 2 |
| Maturidade | profiles Current e Target, 10 lacunas de 11 subcategorias | 3 |
| Plano | 13 itens nos 8 domínios, itens de criptografia com os cinco detalhes | 3, 4, 5 |

A matriz de proteção do Entregável 5 traz uma **lacuna deliberada**: as credenciais de serviço só têm item de gestão de chaves, e a matriz a marca como "— (sem item)".

## Exercício manual da interface

Este roteiro usa somente o diretório sanitizado da demonstração e **não registra aceite visual** por si só:

1. Abra `dfir-app` e alterne entre uma janela larga (por exemplo, 1280×820) e uma compacta (960×600). Confirme que **Evidências** e **Incidentes** continuam acessíveis.
2. Abra **Preferências da interface**, percorra Light, Dark, Vento Áureo e Custom e observe a pré-visualização. Selecione **Cancelar** e confirme que o tema salvo foi restaurado.
3. Reabra a janela, escolha um preset, ative **Reduzir movimento** e salve. Reabra a aplicação no mesmo `DFIR_DATA_ROOT` sanitizado e confirme que as preferências persistiram.
4. Se o seletor de arquivos for exercitado, registre separadamente backend e comportamento observado. A meta de latência nativa e outros backends não são validados por este roteiro. `EVD-060` cobre escolha real e cancelamento no Zenity deste host, mas não substitui o exercício manual desta demonstração.

## O que já foi verificado

`just e2e caminho/arquivo.evtx` roda este roteiro com o engine e o `.evtx` reais e confere, entre outros pontos: contagens do cenário, rótulo DEMO nos cinco documentos, hashes do `manifest.json` contra os arquivos, tabelas bem formadas, ausência de seção vazia, mascaramento (IP, e-mail e nome fictícios somem; marcadores aparecem), geração sem máscara, determinismo byte a byte entre duas gerações, descarte da evidência (os documentos continuam citando hash e custódia) e `audit-verify`. Os testes unitários de `service/demo.rs` cobrem o mesmo sem engine.

Em 2026-11-21, `EVD-057` registrou uma execução isolada de Preferências com Customizado, base Vento Áureo, matiz 160, croma 0.3 e Reduced Motion ativo. Persistência, preview sem tons intermediários na amostra local e cancelamento com restauração pixel a pixel foram confirmados; o Super VL avaliou as duas janelas como consistentes. Isso não valida latência ou desempenho de repaint. Em 2026-11-23, `EVD-050` confirmou o fluxo X11 com importação manual, jobs, timeline e abrir/cancelar seletor; `EVD-060` acrescentou seleção real no Zenity e regressão manual; `EVD-061` confirmou as geometrias 960×600, 1024×640 e 1280×820 com navegação nova no modo compacto e zero erros X11. Nenhum deles declara aceite visual desta demonstração, desempenho de repaint ou outros backends.

O script aceita **qualquer** EVTX e não exige um arquivo específico, mas algumas etapas dependem do conteúdo: por exemplo, a revisão de candidatos precisa que as regras encontrem pelo menos duas hipóteses, e o roteiro de filtros usa os Event IDs realmente presentes. Se o arquivo não tiver esses sinais, a etapa correspondente imprime `FAIL` honesto e o script continua (não estoura mais com `IndexError`); o resultado **110/110** documentado usa o `security.evtx` de referência (2 261 eventos, 6 candidatos). O EVTX `CA_hashdump_4663_4656_lsass_access.evtx` citado no registro de 2026-11-14 é uma amostra pública de 2 eventos (4663/4656): ele exercita o pipeline, mas não a detecção de criação de conta. A procedência desse arquivo não foi estabelecida; **não** o distribua como parte do pacote.

## Como alterar o cenário

Edite `src/service/demo.rs`, **suba `DEMO_DATASET_VERSION`** e ajuste as contagens esperadas em `service/demo.rs` (testes) e em `scripts/e2e_headless.py` (`DEMO_COUNTS`). Texto novo continua sendo sintético e rotulado; não use dado real.
