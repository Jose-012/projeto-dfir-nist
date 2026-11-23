<!-- doc-type: reference | status: current -->
# Operação: instalação de referência, backup e atualização manual (F1-12, RF-030, RF-032)

Documento do operador. **Os comandos abaixo são de referência e não foram executados neste ambiente de desenvolvimento**; confira a sintaxe na documentação da sua versão do sistema.

## 1. Instalação de referência

1. Compile (ou receba) `dfir-app` e `dfir-engine` e deixe-os **na mesma pasta**; sem `DFIR_ENGINE_PATH`, o app procura o engine ao lado de si.
2. Escolha uma pasta de dados num volume **protegido** (seção 2) e aponte `DFIR_DATA_ROOT` para ela. A partida registra a pasta efetiva (`resolved evidence/data storage root`).
3. A aplicação não abre porta de rede (ADR-0017) e não precisa de Internet para operar. O que usa Internet é só obter as dependências e as ferramentas, antes.
4. Confira com `dfir-app diagnostics` (versões, contagens, disco; sem nomes de arquivo) e `dfir-app audit-verify`.

## 2. Proteção do volume (RF-030)

O aplicativo **não criptografa a pasta de dados por conta própria**: evidências, banco e exportações ficam em arquivos comuns. A proteção em repouso vem do volume.

- **Linux (LUKS2):** `cryptsetup luksFormat --type luks2 /dev/<partição>`, `cryptsetup open /dev/<partição> dfir-dados`, `mkfs.ext4 /dev/mapper/dfir-dados`, montar e usar uma pasta dentro dele como `DFIR_DATA_ROOT`. Guarde a frase-senha e o cabeçalho LUKS (`cryptsetup luksHeaderBackup`) fora da máquina.
- **Windows (BitLocker):** ligue o BitLocker no volume que contém a pasta de dados (Painel de Controle ou `manage-bde -on <unidade>: -RecoveryPassword`), guarde a chave de recuperação **fora da máquina** e confira com `manage-bde -status <unidade>:`.
- As **exportações** (`deliverables`) saem mascaradas por padrão, mas o arquivo gerado continua sendo dado do caso: grave-o também em volume protegido. Gerar sem máscara (`--no-redaction`) é escolha explícita e fica na auditoria.

## 3. Backup e atualização (RF-032)

### 3.1 O que o aplicativo faz sozinho (ADR-0026)

Ao abrir uma build nova sobre um banco **que já tem dados** e migrations pendentes, o aplicativo, **antes de migrar**:

1. copia o banco com `VACUUM INTO` para `backups/pre-migration-<UTC>-v<de>-to-v<para>/dfir.sqlite`;
2. verifica a cópia (SHA-256, `integrity_check`, chaves estrangeiras, mesmas migrations aplicadas, **mesma contagem de linhas de cada tabela**);
3. grava `manifest.json` ao lado e só então publica a pasta;
4. aplica as migrations e registra `DB_PRE_MIGRATION_BACKUP` na auditoria.

Se a cópia ou a verificação falhar (inclusive por falta de espaço: tamanho do banco mais `DFIR_MIN_FREE_DISK_BYTES`), **a migration não é aplicada** e o aplicativo mostra o erro; o banco original segue como estava. Banco novo e banco já atualizado não geram cópia.

Para reverificar uma cópia depois (hash antes de abrir o arquivo, depois o SQLite):

```bash
dfir-app backup-verify pre-migration-20261027T120000Z-v11-to-v12   # nome da pasta em backups/, ou caminho
```

**O que isto não cobre:** `evidence/` (migrations não tocam) e o resto da pasta de dados; backups antigos **não são apagados** pelo aplicativo; restaurar é manual.

### 3.1.1 Política de retenção de `backups/` (decisão do operador, 2026-11-14)

O aplicativo **nunca apaga** nada de `backups/` sozinho. A política operacional é do operador:

- cada pasta `pre-migration-<UTC>-v<de>-to-v<para>/` fica até o operador removê-la; uma boa regra é manter **as duas últimas** e só apagar depois que `dfir-app backup-verify <pasta>` passar e a nova build tiver rodado `audit-verify` limpo sobre os dados reais;
- antes de apagar, confira o espaço: `du -sh ~/.local/share/dfir/storage/backups/` (cada cópia tem o tamanho aproximado do banco, porque é `VACUUM INTO`);
- a limpeza é sempre manual (`rm -rf` na pasta específica, nunca um glob amplo), depois de `backup-verify`.

### 3.2 Antes de atualizar (continua valendo)

A cópia automática protege o banco **contra a migration**; não substitui o backup da pasta de dados, que protege contra disco, erro humano e o resto.

1. Feche o aplicativo e qualquer comando headless em andamento.
2. `dfir-app audit-verify` (na build antiga): precisa terminar sem erro.
3. Copie a **pasta de dados inteira** (`db/`, `evidence/`, `exports/`, `backups/`...) para outro volume, preservando datas (`cp -a` no Linux). O banco é `db/dfir.sqlite`; se existirem `dfir.sqlite-wal` e `dfir.sqlite-shm`, copie os três juntos.
4. Verifique a cópia: aponte `DFIR_DATA_ROOT` para ela e rode `dfir-app audit-verify` **com a build antiga**; compare também `sha256sum` do `dfir.sqlite` original e da cópia (com o app fechado).
5. Instale a build nova **em outra pasta**, sem sobrescrever a antiga.

Atualizar: abra a build nova com os dados reais (a cópia automática acontece na partida) e rode `dfir-app audit-verify`.

### 3.3 Voltar atrás (rollback)

Migrations **não têm reversão**. Feche o aplicativo e restaure o banco: ou a pasta de dados da cópia do passo 3.2, ou, só o banco, o `dfir.sqlite` de `backups/pre-migration-.../` (apague `dfir.sqlite-wal` e `dfir.sqlite-shm` do banco vivo antes de substituí-lo) e use a build antiga. Rode `dfir-app backup-verify` na cópia **antes** de restaurar. O que foi registrado depois da atualização se perde; exporte antes, se importar.

## 3A. Contenção do engine (ADR-0024)

O engine lê o log, que é entrada hostil. No **Linux** ele se contém sozinho antes de ler a evidência (`no_new_privs`, Landlock, seccomp): só lê o arquivo do job, não abre rede, não executa outro programa. O padrão é `required`: se uma camada não puder ser aplicada, **o engine recusa analisar** (`SANDBOX_UNAVAILABLE`) em vez de rodar sem ela.

| Variável | Efeito |
|---|---|
| `DFIR_ENGINE_SANDBOX=required` | padrão no Linux; falha fechada |
| `DFIR_ENGINE_SANDBOX=best-effort` | padrão no Windows e outros; aplica o que existir, o job registra `SANDBOX_NOT_FULLY_APPLIED` com o estado |
| `DFIR_ENGINE_SANDBOX=off` | só depuração; fica registrado no job |
| qualquer outro valor | **recusa a partida do aplicativo** (um erro de digitação não pode parecer "ligado") |

Quando `required` bloqueia no Linux: kernel sem Landlock (anterior ao 5.13 ou com o módulo desligado; confira `cat /sys/kernel/security/lsm`) ou **aarch64** (o seccomp só existe para x86_64). Escolher `best-effort` é decisão sua, consciente, e o estado real continua aparecendo no job.

**No Windows** há Job Object (limite de memória, nenhum processo filho, sem acesso à interface) e **não há** confinamento de arquivo nem de rede; todo job traz o aviso. Esse caminho **nunca foi executado**: rode `dfir-app engine-smoke` e uma análise de teste antes de confiar nele.

Isto contém uma falha do parser; **não** protege de tudo. Para evidência de origem desconhecida, analise numa **máquina ou VM dedicada, offline** (spec §29.3): é a camada que não depende de nenhum código.

Para ver o estado do último job: o aviso `SANDBOX_NOT_FULLY_APPLIED` aparece em `dfir-app results <JOB_ID>` (campo `warnings`) e o estado está em `ANALYSIS_COMPLETED` na auditoria.

## 3B. Filtro estruturado da timeline

O campo de filtro aceita texto livre (título, descrição, provedor, canal) e termos `campo:valor`, que se combinam por **E**; vírgula num mesmo termo é **OU**; aspas agrupam.

| Termo | Casa com |
|---|---|
| `ref:record:5,record:9` | referência bruta exata do evento de origem |
| `event:4624,4625` | Event ID exato |
| `user:fsir` | conta contém o texto |
| `host:DC01` | máquina contém o texto |
| `ip:10.0.0.5` | IP de origem **ou** destino, exato (`src:`/`dst:` para um lado só) |
| `proc:powershell` | processo contém o texto |
| `action:logon` | ação normalizada (`logon`, `logon_failed`, `service_installed`...) |
| `outcome:failure` | `success` ou `failure` |
| `provider:` / `channel:` | provedor / canal contém o texto |
| `from:2026-03-01` `to:2026-03-02` | intervalo em UTC; `to:` com data inclui o dia inteiro |

Exemplo: `event:4625 user:admin from:2026-03-01`. Uma data inválida é **erro**, não busca de texto. Na linha de comando: `dfir-app results <JOB_ID> --filter "event:4625 user:admin"`.

Em um candidato de detecção com referências preservadas, **Ver eventos na timeline** abre e expande a timeline da mesma evidência com `ref:` e os identificadores exatos. A consulta resolve o último job EVTX concluído, não o job da detecção. Referências truncadas são sinalizadas como seleção parcial; candidato sem referências não oferece a ação. Navegar não confirma nem rejeita o candidato.

A trilha textual no topo do painel central mostra `Evidências > <nome lógico>` ou `Incidentes > <incidente> > <aba>`. Ela deriva da seleção que já controla a tela e converge no quadro seguinte às ações, sem criar uma segunda navegação nem depender de imagens. Esse comportamento foi testado sem janela; não é uma declaração de suporte AT-SPI nem de fluxo X11 verificado.

## 3C. Detecção por regras e revisão de candidatos

Na tela de evidência (EVTX), **Detectar** roda as regras (`docs/DETECTION_RULES.md`) e abre o painel "Detecções por regras". Na linha de comando:

```bash
dfir-app detect <EVIDENCE_ID>                       # roda as regras; grava candidatos PENDENTES
dfir-app candidates <EVIDENCE_ID> [--status PENDING]
dfir-app candidate-confirm <CANDIDATE_ID> --incident <INCIDENT_ID> [--note "..."]
dfir-app candidate-reject  <CANDIDATE_ID> --reason "motivo"
```

**Um candidato é uma hipótese.** Ele traz a regra e a versão, a técnica ATT&CK sugerida, os eventos de origem (`record:N`), os IOCs vistos e o que a regra **não** prova. **Confirmar** cria no incidente o mapeamento ATT&CK, o achado e os IOCs; **rejeitar** exige motivo. A decisão é final. Rodar de novo não duplica nem desfaz decisões. O Entregável 1 (§4.3) diz quantos candidatos ficaram **pendentes**: pendente não é achado.

**A ausência de candidato não prova ausência de ataque.** A regra só enxerga o que o log registrou. Para as regras de processo (linha de comando), de compartilhamento e de conexão de saída existirem, a auditoria correspondente precisa estar **ligada no Windows** (criação de processo com linha de comando — 4688 —, compartilhamento de arquivos — 5140/5145 —, plataforma de filtragem — 5156 — ou Sysmon). Sem elas o log não tem o sinal: isso é **lacuna de fonte**, registre-a na aba "Lacunas".

## 3E. Vínculo evidência↔incidente, custódia e auditoria (F1-19/F1-24, 2026-11-15)

Na GUI, o detalhe da evidência tem o painel "Vínculo com incidente e custódia"; a aba Incidentes ganhou as abas **Evidências** (evidências vigentes daquele incidente, sem janela de tempo) e **Auditoria** (eventos com prefixo de hash encadeado). Na linha de comando:

```bash
dfir-app link-evidence <EVIDENCE_ID> --incident <INCIDENT_ID>   # idempotente; grava EVIDENCE_LINKED_TO_INCIDENT
dfir-app incident-evidence <INCIDENT_ID>                        # evidências vigentes do incidente
dfir-app custody <EVIDENCE_ID>                                  # cadeia de custódia (append-only)
dfir-app audit [--limit N]                                      # eventos de auditoria com hash encadeado
```

Confirmar um candidato (`candidate-confirm`) também vincula a evidência ao incidente. A destruição de evidência preserva custódia e auditoria (F1-21).

Desde a migration 0013 (F1-36/ADR-0030), `custody_state` mantém por evidência a contagem e o hash da cabeça, atualizados na mesma transação de cada append. A verificação falha fechado se a âncora estiver ausente ou divergir da cadeia, inclusive quando a cauda é removida. O upgrade preenche âncoras a partir das entradas existentes, sem reescrevê-las. Esse backfill não pode descobrir uma remoção que já tenha ocorrido antes da migration; também não impede um atacante com escrita no banco de alterar coordenadamente entradas e âncora de forma consistente. A âncora é uma guarda local adicional, não uma assinatura ou uma âncora externa de confiança.

## 3D. Diálogo nativo de arquivos no Linux (GUI-01)

O aplicativo **não precisa** de portal nem de `zenity` para iniciar e analisar evidência. O
seletor "Importar evidência…" usa o `file_dialog.rs` que escolhe o backend via
variável de ambiente `DFIR_FILE_DIALOG_BACKEND`:

- `auto` (padrão): no Linux, sonda curta e cacheada (executa `zenity --version` **fora da UI**) escolhe `zenity` quando disponível; caso contrário, cai para `rfd`. Nas demais plataformas, sempre `rfd`.
- `zenity`: sempre o `zenity` direto via `std::process::Command` (sem shell, **sem timeout enquanto o operador escolhe**, saída limitada). Em outra plataforma, devolve `Picked::Unavailable` com diagnóstico explícito.
- `rfd`: sempre o caminho legado `rfd` (portal primeiro no Linux), com a limitação residual de não distinguir cancelamento de falha do seletor.

Um valor **inválido** devolve erro específico (mesma régua de `DFIR_ENGINE_SANDBOX`), para que erro de digitação não pareça configuração padrão.

No caminho `rfd` legado, quando o portal não responde (D-Bus ~25 s), o seletor devolve "nada escolhido" por limitação da API; o Zenity distingue indisponibilidade e falha. A janela **não congela**, porque o diálogo roda em uma thread própria (`src/gui/file_dialog.rs`).

Instalação de referência em Debian/Ubuntu (a versão exata depende da distribuição):

```bash
sudo apt install xdg-desktop-portal xdg-desktop-portal-gtk zenity
```

Depois, encerre e abra a sessão novamente (ou reinicie o serviço do portal). Para
diagnosticar:

```bash
which zenity
echo "$XDG_CURRENT_DESKTOP"
systemctl --user status xdg-desktop-portal
busctl --user list | grep -i portal
```

Se o seletor não abrir, há dois caminhos que não dependem dele: digitar o caminho no campo
"Ou informe o caminho:" e usar "Importar caminho", ou **arrastar o arquivo** para a janela.
O aplicativo trata os três caminhos pela mesma validação (`request_import`): caminho vazio,
pasta ou importação já em andamento são recusados com uma mensagem na tela; vários arquivos
soltos importam o primeiro e avisam quantos ficaram de fora. O `rfd` legado não distingue
"cancelado" de "seletor indisponível"; por isso a mensagem orienta os caminhos alternativos,
em vez de fingir que a importação ocorreu.

## 3F. Entregáveis e SVG do caminho crítico (F1-39/ADR-0030)

Além dos cinco documentos Markdown, a geração grava `caminho-critico.svg` e o inclui no `manifest.json`: são seis entradas em `documents` e sete arquivos no total, contando o manifest. O SVG ordena os estágios por sequência e ID, valida as arestas, neutraliza caracteres invisíveis e escapa texto para XML 1.0. Não contém JavaScript nem recursos externos; `<title>` e `<desc>` preservam uma alternativa textual integral, inclusive quando não há estágios. Payload inválido, ID duplicado ou aresta sem origem/destino fazem a geração falhar com diagnóstico, em vez de produzir um grafo incompleto.

A pré-visualização continua mostrando os cinco Markdown e não grava nem assina arquivos. O SVG é uma representação exportada dos dados atuais; seu teste headless não comprova renderização em navegador, X11 ou tecnologia assistiva.

## 3G. Cancelamento explícito de análise (F1-38/ADR-0030)

Na tela da evidência, **Cancelar análise** aparece enquanto existe análise vigente. Na linha de comando:

```bash
dfir-app cancel-analysis <EVIDENCE_ID>   # cancela o job vigente mais recente
dfir-app results <JOB_ID>                # confirma status CANCELLED, sem timeline parcial
```

O serviço `cancel_job` também aceita o ID específico do job. A requisição persiste `CANCELLED`, grava `ANALYSIS_CANCELLED` e custódia, notifica o worker local e é observada por outra instância através do banco. O engine é morto e recolhido. Repetir o cancelamento é seguro; se o job já terminou, `COMPLETED` é preservado. Falhas ou conclusões tardias não substituem o estado cancelado, e eventos/resultados parciais não são publicados.

## 3H. Assinatura e verificação offline (F1-40/ADR-0030)

Crie e proteja a chave privada fora de `DFIR_DATA_ROOT` e fora da pasta exportada. O arquivo deve conter exatamente 32 bytes em 64 caracteres hexadecimais, com no máximo uma quebra de linha final. O aplicativo não gera essa chave. Para um arquivo de 32 bytes aleatórios já protegido, uma conversão de engenharia possível é `od -An -tx1 -v CHAVE.bin | tr -d ' \n' > CHAVE.hex`; não execute isso sobre segredos reais em logs ou diretórios compartilhados.

```bash
dfir-app deliverables <INCIDENTE> --out ./saida --sign-with /caminho/externo/CHAVE.hex
# stdout informa somente public_key e public_key_id

dfir-app verify-deliverables ./saida --public-key <64_HEX_DA_CHAVE_PUBLICA>
```

`manifest.json.sig` usa o schema `dfir-deliverables-signature/1`. A assinatura Ed25519 cobre os 32 bytes do SHA-256 de `manifest.json`; `public_key_id` é `sha256:<hash-da-chave-pública>`. A verificação exige a chave pública esperada e confere schema, algoritmo, assinatura, manifesto e tamanho/SHA-256/BLAKE3 dos cinco Markdown e do SVG. Ela falha com chave errada, artefato adulterado ou ausente. Não há rede nem confiança em chave fornecida somente pelo envelope.

A geração assinada reverifica antes de auditar. Preview não assina; gerar novamente sem `--sign-with` remove a assinatura antiga para não deixar um envelope inválido junto de arquivos novos.

## 3I. Linhagem, Incidentes e plano de hardening

Em **Evidências**, abra "Resultados analíticos (YARA)" e use **Inspecionar linhagem** no resultado desejado. Em **Incidentes > Plano**, a mesma ação está disponível em cada item. A consulta é somente leitura e valida o escopo: resultado selecionado, histórico completo da `correlation_key`, versão vigente e relações de saída. Não há grafo global nem edição de relações. JSON inválido, relação desconhecida ou alvo ausente produz diagnóstico específico, não uma lista parcial silenciosa. Após uma falha, clicar novamente repete a consulta; trocar de evidência ou incidente limpa o contexto anterior.

A lista de **Incidentes** começa sem consulta automática: **Carregar lista** inicia o carregamento e uma falha oferece **Tentar novamente**. A contagem aparece somente quando os dados estão prontos; `Idle` não afirma que existem zero incidentes.

Todo item novo do plano exige pré-condição, pós-condição e verificação manual, além de **Fonte** e **Confiança**. O formulário começa deliberadamente em `NOT_VERIFIED`/`LOW`; o analista deve elevar esses estados somente com base no que registrou. Registros antigos sem `source`/`confidence` continuam legíveis e aparecem como "não informado (registro legado)". Esses campos organizam recomendações e o Entregável 4; **nenhum hardening é executado automaticamente** nem há eficácia declarada sem verificação manual.

## 3J. Smoke do fluxo nativo X11

`scripts/native_x11_smoke.py` é ferramenta de Engineering/QA para uma sessão X11 disponível. Ele exige Python com `Xlib`, `xclip`, `zenity`, o binário debug e a fixture sanitizada já presentes; não instala dependências, não usa screenshots nem AT-SPI. Use uma raiz de dados nova e um caminho de resultado fora do repositório:

```bash
cargo build --locked --bin dfir-app
python3 scripts/native_x11_smoke.py \
  --data-root <DIRETORIO_VAZIO_EM_SCRATCH> \
  --results <RESULTADOS_JSON_EM_SCRATCH>
```

O script recusa uma raiz não vazia e janelas DFIR/seletores preexistentes. Ele injeta ações por XTEST, mas o aceite vem de pós-condições independentes: enums de tela no log sanitizado, evidência e jobs no SQLite isolado, eventos na `timeline_index`, página de timeline com `has_page:true` e ciclo de vida da janela/processo Zenity. A timeline é localizada por essas pós-condições, com rolagem confinada à janela; o retorno ao topo precede o seletor. O cleanup afeta somente os processos iniciados pelo smoke.

O fluxo padrão importa `tests/fixtures/senai-demo.evtx` pelo **campo manual**. Com `--import-method zenity`, a importação usa o **seletor real**: Ctrl+L no Zenity, caminho colado, Enter, outcome `path` e diálogo fechado antes da leitura do SQLite. Nos dois modos, o smoke executa EVTX e detecção, exige eventos e candidatos não vazios, abre uma página de timeline não vazia e depois abre/cancela outro seletor, conferindo que o catálogo permanece com uma evidência. Coordenadas podem ser ajustadas pelas opções `--help`; qualquer timeout, candidato vazio ou pós-condição ausente produz `flow: FAIL`. IDs, contagens e hash da fixture entram no relatório, não nomes/caminhos de evidências ou conteúdo de eventos. `EVD-060` registra os dois modos neste host, sem promover outros backends, AT-SPI ou meta de latência.

Com `--exercise-resize`, o harness exige geometria exata 960×600, 1024×640 e 1280×820, captura erros do protocolo X11 e confirma navegação nova no modo compacto por offsets de log. Ao final, exige processo vivo e ausência de `BadMatch`/panic. `EVD-061` registra esse fluxo com seleção Zenity, a regressão manual sem a flag e um controle negativo em cópia descartável que rejeitou divergência de 1 px. Isso não substitui inspeção estética nem mede desempenho de repaint.

## 4. O que este documento não cobre

Bundle assinado, preflight e health/smoke automáticos, e `.dfirpkg` são Fase 4 (ADR-0018). Importar CSV/JSONL (rede, SIEM, dispositivos de segurança) foi adiado por decisão do operador (ADR-0025).

## 5. Amostras de evidência e fixtures (F1-16) — política de distribuição (decisão do operador, 2026-11-14)

- **O pacote não distribui nenhum `.evtx`** (`.gitignore` exclui `/*.evtx` da raiz; `tests/fixtures/README.md` registra a política). O e2e e a demo recebem o caminho do operador: `just e2e <arquivo.evtx>`, `just demo <arquivo.evtx>`.
- A amostra usada nas verificações registradas é o `security.evtx` público do repositório `omerbenamram/evtx` (SHA-256 `5f29a03c…9a4d8d`, 2 166 784 bytes; ver `tests/fixtures/README.md` e `docs/DEMO.md`). A procedência do arquivo em si não é documentada nesse repositório, então ele **não entra no pacote** — quem roda os gates baixa o arquivo por conta e registra o próprio SHA-256 junto da evidência.
- **Fixtures sanitizadas versionadas (F1-16) CONCLUÍDO em 2026-11-15 (`EVD-011`)**: `tests/fixtures/senai-demo.evtx` é um `.evtx` mínimo gerado e sanitizado pelo próprio projeto, determinístico (mesmo SHA-256 a cada geração: `256a655a7e259908e1dfd23b914abf27b7ecb8fd98c0b3f73033336f6ed448be`) e versionado no repositório. O e2e roda sem arquivo externo: `just e2e tests/fixtures/senai-demo.evtx` (110/110 em 2026-11-15, `EVD-011`). O pacote continua não distribuindo o `security.evtx` do operador; a fixture versionada é a única amostra distribuída. Os gates que precisam de EVTX real de campo ficam cobertos por `EVD-002`/`EVD-006` (arquivo fornecido pelo operador) e as regras de detecção que a fixture não exercita (4625/4688/7045/1102) têm fixtures sintéticas em `engine/src/detect.rs`.
