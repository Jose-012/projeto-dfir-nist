# Fixtures

Fixtures reais/sanitizadas entram quando a Fase 1 implementar o pipeline de evidência.
O smoke de protocolo da Fase 0 não depende de fixture EVTX real; ele usa uma referência `.evtx` deliberadamente ausente para validar o caminho fail-closed e a mensagem `INPUT_UNAVAILABLE`.

## EVTX de amostra da demonstração e do e2e (F1-16, parcial)

O projeto **não distribui** nenhum `.evtx`. O e2e (`just e2e <arquivo>`) e a demonstração (`just demo <arquivo>`, `docs/DEMO.md`) recebem o caminho do operador. O arquivo usado nas verificações:

| Campo | Valor |
|---|---|
| Nome | `security.evtx` |
| Origem | repositório `omerbenamram/evtx`, pasta `samples/` (baixado em 2026-10-22) |
| Tamanho | 2 166 784 bytes |
| SHA-256 | `5f29a03cf8c1b4bfbd82c4074ccc81ecfe85d7ee15375afc352ed5025c9a4d8d` |
| Licença | o repositório é MIT ou Apache-2.0; **a procedência do arquivo em si não é documentada lá** (não verificada). Por isso ele não entra no pacote |
| Conteúdo | log de segurança comum, **sem ataque**; a varredura YARA-X não encontra nada nele (esperado). As regras de detecção geram **6 candidatos**, medidos em 2026-10-27: `R-PERS-001` ×2 (contas `fsir` e `archir` criadas), `R-PRIV-002` ×2 (as duas contas adicionadas ao grupo Administrators) e `R-PRIV-001` ×2 (`fsir`, 41 eventos, e `archir`, 2 eventos, com SeDebug/SeTcb/SeLoadDriver). São o que o arquivo contém (a preparação de uma máquina de teste), não um ataque. Não há 4625, 4688, 7045 nem 1102 nele, então as regras de rajada, processo, serviço e log apagado **não são exercitadas por este arquivo**: elas têm fixtures sintéticas em `engine/src/detect.rs` |

Qualquer outro `.evtx` serve para a demonstração: o cenário sintético não depende do conteúdo. O e2e confere o SHA-256 do arquivo informado contra o que a importação grava, não contra o valor acima.

## O `.evtx` de teste é fornecido pelo operador (Gap origem: AUS-07, 2026-11-04)

O pacote não contém nenhum arquivo `.evtx` (nenhum no repositório) e o projeto não distribui amostras. O e2e (`just e2e <arquivo>`), a demonstração (`just demo <arquivo>`) e o teste de integração que exige uma fixture real dependem de um `.evtx` que **o operador fornece** no ambiente de execução. Enquanto o operador não fornecer o arquivo, esses gates permanecem `PENDING-HUMAN` (ver `docs/RELEASE_GATE.md` e `docs/EVIDENCE_REGISTER.md`); a fixture sanitizada e versionada é o ticket F1-16 (`PLANNED`).

O arquivo usado nas verificações registradas é `security.evtx`, com SHA-256 `5f29a03cf8c1b4bfbd82c4074ccc81ecfe85d7ee15375afc352ed5025c9a4d8d` (o mesmo valor está em `docs/DEMO.md`). Quem usar outro arquivo deve registrar o próprio SHA-256 junto da evidência: o e2e confere o hash do arquivo informado contra o gravado pela importação, não contra o valor acima.
