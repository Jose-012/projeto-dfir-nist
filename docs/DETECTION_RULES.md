<!-- doc-type: reference | status: current -->
# Regras de detecção do engine (F1-28)

Gerado de `dfir-engine --list-rules` (conjunto de regras **2026.11.1**). ADR-0025. Um teste do engine confere que este arquivo cita todas as regras e a versão do conjunto; ao criar, remover ou alterar o limiar de uma regra, mude `RULE_SET_VERSION` em `engine/src/detect.rs` e regenere esta tabela.

**Regra de ouro:** uma regra só produz **candidato**. Nada vira achado, mapeamento ATT&CK ou IOC sem a confirmação do analista. **A ausência de candidato não prova ausência de ataque**: a regra só enxerga o que o log registrou. A confiança de uma regra automática nunca passa de MEDIUM.

| Regra | Tática · técnica | Gravidade | Confiança | O que detecta |
|---|---|---|---|---|
| `R-AUTH-001` v1 — Rajada de falhas de logon | CREDENTIAL_ACCESS · T1110 | MEDIUM | LOW | Dez ou mais falhas de logon (4625) para a mesma origem ou a mesma conta em até 5 minutos. |
| `R-AUTH-002` v1 — Logon bem-sucedido depois de rajada de falhas | INITIAL_ACCESS · T1078 | HIGH | MEDIUM | Um logon bem-sucedido (4624) para a mesma origem ou conta, em até 30 minutos depois de uma rajada de falhas. |
| `R-ACCESS-001` v1 — Logon remoto a partir de IP público | INITIAL_ACCESS · T1078 | MEDIUM | LOW | Logon de rede (tipo 3) ou de área de trabalho remota (tipo 10) bem-sucedido com origem em endereço IP público. |
| `R-LATERAL-001` v1 — Área de trabalho remota a partir de máquina interna | LATERAL_MOVEMENT · T1021 (T1021.001) | MEDIUM | LOW | Logon do tipo 10 (RDP) bem-sucedido com origem em endereço interno, que não o da própria máquina. |
| `R-LATERAL-002` v1 — Credenciais explícitas contra outro servidor | LATERAL_MOVEMENT · T1021 | LOW | LOW | Logon com credenciais explícitas (4648) cujo servidor de destino não é a própria máquina. |
| `R-LATERAL-003` v1 — Acesso a compartilhamento administrativo | LATERAL_MOVEMENT · T1021 (T1021.002) | MEDIUM | LOW | Acesso a ADMIN$ ou a um compartilhamento de unidade (C$, D$...) por origem remota (eventos 5140 e 5145). |
| `R-LATERAL-004` v1 — Serviço típico de execução remota | LATERAL_MOVEMENT · T1021 (T1021.002) | HIGH | MEDIUM | Serviço instalado com nome ou caminho típico de execução remota (PsExec e similares) ou com imagem em ADMIN$/UNC. |
| `R-PRIV-001` v1 — Conta com privilégios sensíveis em uso | PRIVILEGE_ESCALATION · T1078 (T1078.003) | LOW | LOW | Logon de conta de usuário com SeDebug, SeTcb ou SeLoadDriver atribuídos (evento 4672). |
| `R-PRIV-002` v1 — Conta adicionada a grupo privilegiado | PRIVILEGE_ESCALATION · T1098 | HIGH | LOW | Membro adicionado a Administradores, Domain Admins, Enterprise Admins, Schema Admins ou a um grupo de operadores (4728, 4732, 4756). |
| `R-PERS-001` v1 — Conta de usuário criada | PERSISTENCE · T1136 | MEDIUM | LOW | Criação de conta de usuário (evento 4720). |
| `R-PERS-002` v1 — Serviço instalado | PERSISTENCE · T1543 (T1543.003) | LOW | LOW | Instalação de serviço (7045 ou 4697); gravidade sobe quando a imagem está em pasta temporária ou de perfil, ou usa interpretador de comandos. |
| `R-PERS-003` v1 — Tarefa agendada criada | PERSISTENCE · T1053 (T1053.005) | LOW | LOW | Criação de tarefa agendada (evento 4698); gravidade sobe quando o comando usa interpretador de scripts ou pasta temporária. |
| `R-EVAS-001` v1 — Log de eventos apagado | DEFENSE_EVASION · T1070 (T1070.001) | HIGH | MEDIUM | Limpeza do log Security (1102) ou System (104). |
| `R-EVAS-002` v1 — Política de auditoria alterada | DEFENSE_EVASION · T1562 (T1562.002) | MEDIUM | LOW | Alteração da política de auditoria do sistema (evento 4719). |
| `R-EXEC-001` v1 — PowerShell com comando codificado ou download | EXECUTION · T1059 (T1059.001) | MEDIUM | LOW | Linha de comando ou bloco de script do PowerShell com comando codificado em Base64, download de conteúdo ou execução dinâmica. |
| `R-CRED-001` v2 — Indicador de extração de credenciais | CREDENTIAL_ACCESS · T1003 | HIGH | MEDIUM | Linha de comando ou bloco de script com termos típicos de extração de credenciais (mimikatz, sekurlsa, lsadump, cópia de SAM/NTDS, dump de lsass e MiniDumpWriteDump de LSASS). |
| `R-EXFIL-001` v1 — Ferramenta de envio de dados por linha de comando | EXFILTRATION · T1567 | MEDIUM | LOW | Linha de comando com ferramenta ou opção típica de envio de dados para fora (rclone, mega, curl/Invoke-WebRequest com envio, WinSCP, azcopy, aws s3). |
| `R-EXFIL-002` v1 — Conexão de saída de ferramenta de linha de comando para IP público | EXFILTRATION · T1041 | MEDIUM | LOW | Conexão de saída permitida (5156 ou Sysmon 3) para IP público iniciada por PowerShell, cmd, curl, certutil, bitsadmin ou similar. |
| `R-COLL-001` v1 — Compactação de dados por linha de comando | COLLECTION · T1560 (T1560.001) | LOW | LOW | Linha de comando que compacta arquivos (Compress-Archive, 7z, rar, zip), etapa comum antes de enviar dados para fora. |

## O que cada regra não prova

| Regra | Limite do método |
|---|---|
| `R-AUTH-001` | Falhas em rajada também vêm de serviço com senha vencida ou de usuário que esqueceu a senha; a regra não distingue ataque de erro operacional. |
| `R-AUTH-002` | Indica que a tentativa de adivinhar a senha pode ter funcionado; não prova que o logon foi do atacante. |
| `R-ACCESS-001` | Acesso remoto legítimo (VPN com saída pública, teletrabalho) tem o mesmo aspecto; a regra não conhece a lista de origens autorizadas. |
| `R-LATERAL-001` | Administração remota legítima por RDP é idêntica no log; o candidato só aponta quem acessou quem. |
| `R-LATERAL-002` | Comum em tarefas agendadas e mapeamento de unidades; só vale como pista quando combinado com outros candidatos. |
| `R-LATERAL-003` | Ferramentas de inventário e de backup acessam esses compartilhamentos; exige auditoria de compartilhamento ligada para existir. |
| `R-LATERAL-004` | Administradores também usam PsExec; o candidato mostra a instalação, não quem a ordenou nem para quê. |
| `R-PRIV-001` | Todo administrador recebe esses privilégios em todo logon; o candidato lista quem os tem, não prova abuso nem elevação. |
| `R-PRIV-002` | É mudança administrativa válida quando autorizada; a regra não sabe se houve chamado ou aprovação. |
| `R-PERS-001` | Contas são criadas por motivos legítimos todos os dias; vale como pista quando a conta nova aparece em logon ou em grupo privilegiado. |
| `R-PERS-002` | Instalação de software legítimo registra o mesmo evento; o caminho da imagem é a única pista usada. |
| `R-PERS-003` | Sistemas e aplicativos criam tarefas o tempo todo; só o comando da tarefa, quando presente, ajuda a distinguir. |
| `R-EVAS-001` | Administradores limpam logs em manutenção; mas a limpeza também é o gesto típico de quem quer apagar rastros. Exige conferir quem limpou e por quê. |
| `R-EVAS-002` | Mudanças de política são administrativas; a regra não vê se a auditoria foi reduzida ou ampliada. |
| `R-EXEC-001` | Exige auditoria de criação de processo com linha de comando (4688) ou log do PowerShell; scripts de administração também usam Base64. |
| `R-CRED-001` | Casamento por texto: pega a tentativa, inclusive de teste de equipe de segurança, e não pega ferramenta renomeada ou ofuscada. |
| `R-EXFIL-001` | Mostra a tentativa de usar a ferramenta, não o volume nem o destino real dos dados; sem log de rede não há como confirmar a transferência. |
| `R-EXFIL-002` | Pode ser exfiltração, comando e controle ou simples download; sem a carga transferida a regra não distingue. Exige auditoria da plataforma de filtragem (5156) ou Sysmon. |
| `R-COLL-001` | Compactar arquivos é rotina; só ganha peso junto de R-EXFIL-001 ou R-EXFIL-002 na mesma máquina e conta. |

## Cobertura dos pedidos da demanda

| Pedido | Regras | Depende de auditoria ligada no Windows |
|---|---|---|
| Vetor inicial | `R-ACCESS-001`, `R-AUTH-002`, `R-AUTH-001` | logon (4624/4625) |
| Movimentação lateral | `R-LATERAL-001..004` | logon, credenciais explícitas (4648), compartilhamento (5140/5145), serviço (7045/4697) |
| Escalonamento de privilégios | `R-PRIV-001`, `R-PRIV-002` | privilégios especiais (4672), gerenciamento de grupo (4728/4732/4756) |
| Tentativa de exfiltração | `R-EXFIL-001`, `R-EXFIL-002`, `R-COLL-001` | criação de processo com linha de comando (4688), plataforma de filtragem (5156) ou Sysmon, log do PowerShell (4104) |

Sem essas auditorias o log não contém o sinal e a regra não tem o que avaliar. Isso é uma **lacuna de fonte** (spec RF-054), não ausência de ataque.

## Limiares fixos

- Rajada de falhas de logon: 10 falhas (4625) em até 5 minutos, por origem ou por conta; origem com 3 ou mais contas distintas vira *password spraying* (T1110.003).
- Sucesso depois de rajada: até 30 minutos depois da última falha, mesma origem ou mesma conta.
- IP público: fora de faixas privadas, loopback, link local, CGNAT, multicast, documentação (RFC 5737/3849), benchmark e reservadas.
- Tetos: 5 000 candidatos por execução, 50 000 chaves de rajada, 20 referências de evento e 20 IOCs por candidato.
