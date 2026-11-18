<!-- doc-type: decision | status: accepted -->
# ADR-0024 — Contenção do engine (F1-22) e terminologia de "sandbox"

## Status

ACEITA — 2026-10-27, em resposta ao pedido do operador de concluir as etapas da análise de contenção do engine ("do mais barato ao mais caro") e resolver a inconsistência de terminologia que ela apontou. Complementa a ADR-0018 (que deixou o Job Object e o limite de memória no Windows fora do MVP) e a ADR-0007 (que removeu o bubblewrap em favor de processo separado e validação de caminho). **Não** a substitui: o isolamento por processo continua sendo a base; esta ADR acrescenta camadas por cima dela.

## Contexto

1. **O risco.** O log é entrada hostil e o parser (`evtx`, YARA-X) pode ter falha. O engine roda com os privilégios do usuário; uma falha explorável alcançaria o banco, as chaves e a rede do app.
2. **O que existia (v6.5.0, medido no código):** processo separado, `RLIMIT_AS` (só Unix), timeout e encerramento, caminhos validados pelo próprio engine, interface nativa sem HTML/JS. Isso contém travamento e esgotamento de recursos, **não código malicioso**. Faltavam `env_clear`, o arquivo único (`DFIR_ENGINE_INPUT_FILE` era definida e nunca lida), `kill_on_drop`, qualquer contenção do kernel e, no Windows, qualquer limite.
3. **Inconsistência de documentação.** O cabeçalho da especificação diz que a demanda não exige sandbox de SO, mas `RF-036`, `CONVENTIONS` e o prompt de continuidade falavam em "falhar fechado sem sandbox". Na prática "sandbox" significava "processo separado". Dois leitores chegavam a conclusões opostas.
4. **A demanda não exige sandbox de SO** (verificado em `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md`); a restrição RB-006 (confidencialidade na investigação) é atendida por execução local e máscara. A contenção é decisão de engenharia sobre o risco do item 1, não requisito da demanda.

## Decisão

### D-1 — Vocabulário (vale para todos os documentos)

| Termo | Significa | Onde vale |
|---|---|---|
| **Isolamento** | N1: processo separado, ambiente reduzido, arquivo único, supervisão (`RLIMIT_AS` em Unix, timeout, `kill_on_drop`) | todos os sistemas |
| **Sandbox** | N2: contenção pelo sistema operacional **aplicada pelo próprio engine antes de ler a entrada** | Linux (completa); Windows (parcial: Job Object) |
| **Máquina dedicada** | N3: analisar numa VM/máquina offline (spec §29.3) | operacional, sem código |

"Falhar fechado" passa a significar: **se o engine não puder ser criado ou supervisionado, ou se a política exigir uma camada que não pôde ser aplicada, nenhuma evidência é lida.** Não existe queda silenciosa.

### D-2 — N1 em todos os sistemas (F1-22)

- `env_clear()` no processo do engine. Em Unix ele não herda nada; no Windows recebe só `SystemRoot`, `SystemDrive`, `TEMP`, `TMP` e um `PATH` refeito apontando para `System32`. As variáveis `DFIR_ENGINE_*` são definidas à parte. (`reduced_environment`, testado nas duas formas; em Unix, também com um processo real.)
- **Arquivo único imposto:** o engine só aceita o arquivo de `DFIR_ENGINE_INPUT_FILE` (canonicalizado antes do sandbox). Qualquer outro caminho dentro da raiz volta `INPUT_REJECTED`. Sem a variável (uso manual/QA, que o app nunca faz) vale o comportamento antigo por raiz, e o sandbox libera a leitura da raiz inteira.
- `kill_on_drop(true)`: soltar o handle sem `kill` explícito encerra o filho (teste com processo real).

### D-3 — N2 no Linux: Landlock + seccomp + `no_new_privs`

Aplicados no `main()` do engine, **antes de qualquer thread e de qualquer leitura da evidência** (valem para a thread que chama e as que ela criar, inclusive os workers do tokio):

- **Landlock** (ABI negociada, medida: 7): o engine lê só o arquivo do job (no smoke de processo, escreve só na pasta de saída do job). Qualquer outro `open` ou `readdir` falha com `EACCES`; criar arquivo também. Com ABI ≥ 4 nega bind/connect TCP; com ABI ≥ 6 isola sockets Unix abstratos e sinais.
- **seccomp** (x86_64; lista de **negação**, 44 chamadas): `socket`, `connect`, `bind`, `listen`, `accept*`, `execve*`, `ptrace`, `process_vm_*`, `mount` e família, `unshare`/`setns`, `bpf`, `perf_event_open`, módulos do kernel, `io_uring_*`, `open_by_handle_at`, `keyctl` e afins; chamadas da ABI x32 e arquitetura divergente matam o processo. Lista de negação, não de permissão: o engine tem runtime assíncrono, threads e JIT (YARA-X), e uma lista de permissão exaustiva seria frágil demais para o ganho.
- **`socketpair` fica liberado, de propósito.** O driver de sinais do tokio (ligado pela unificação de features quando o workspace é compilado junto com o app) cria um par anônimo no início do runtime, e o engine entrava em pânico sem ele. Foi um defeito real, achado ao rodar o engine de verdade sobre o EVTX real (não aparecia em `cargo test -p dfir-engine`, onde a feature não é unificada). Um par anônimo é IPC dentro do processo; `socket`/`connect`/`bind`/`listen`/`accept` seguem negados.
- `no_new_privs` e `RLIMIT_CORE = 0` (sem core dump com trechos da evidência).
- **Limites conhecidos, não resolvidos:** o filtro clássico do seccomp não enxerga os argumentos de `clone3`, então não bloqueia `clone` com flags de namespace; o seccomp só existe para x86_64 (em aarch64 só o Landlock se aplica, e o estado diz isso); não há limite de CPU além do timeout do supervisor.
- **Prova de que é real, não só aplicado:** `engine --sandbox-selftest` roda **de dentro do processo contido** e tenta o que deve ser negado (ler `/etc/passwd`, listar `/etc`, criar arquivo, `socket` TCP e Unix, `execve`, `ptrace`) e o que deve funcionar (ler o arquivo fixado, criar thread, alocar). O teste de integração `engine/tests/sandbox.rs` exige `EACCES`/`EPERM` onde devido e tem um **controle** (com a política `off` as mesmas tentativas funcionam, então o teste não é vazio). O e2e repete a prova sobre o `security.evtx` real.

### D-4 — N2 no Windows: Job Object (parcial, **não verificado em Windows**)

O engine se coloca num Job Object antes de ler a entrada: teto de memória por processo (o supervisor passa o mesmo valor do `RLIMIT_AS` em `DFIR_ENGINE_MEMORY_LIMIT_BYTES`, já que o Windows não tem `pre_exec`), **no máximo 1 processo ativo** (o engine não cria filhos), sem acesso à interface (área de transferência, desktop, configurações de tela, atoms globais, handles de outras janelas) e encerramento em exceção não tratada. A janela entre iniciar o processo e entrar no job é inócua: o engine só lê a evidência depois de o app enviar o pedido, e o app só o envia depois de `launch_process` retornar.

**O que o Windows não recebe:** confinamento de sistema de arquivos e de rede (exigiria AppContainer ou token restrito, que não foram feitos). Essas camadas ficam `unavailable` e **todo job registra `SANDBOX_NOT_FULLY_APPLIED`**. A política padrão no Windows é `best-effort`.

**Verificação:** o código foi compilado para o alvo `x86_64-pc-windows-gnu` (type-check do engine). **Nunca foi executado em Windows**; é `IMPLEMENTED / PENDING-HUMAN` e a execução fica com o operador.

### D-5 — Política, estado e aviso (RF-036)

`DFIR_ENGINE_SANDBOX` = `required` | `best-effort` | `off`.

| Sistema | Padrão | Efeito |
|---|---|---|
| Linux | `required` | camada indisponível ⇒ o engine responde `SANDBOX_UNAVAILABLE` a todo pedido e não lê nada |
| Windows / outros | `best-effort` | aplica o que existe; o que faltou vira aviso no job |

Valor desconhecido na variável do **app** recusa a partida (`Config`); no **engine** vale `required` (um erro de digitação não desliga a contenção). `off` existe para depuração e fica visível: o estado (`policy=...;nnp=...;landlock=abi7;seccomp=on(44 denied);core=nodump;job=na`) vai do engine para o app num `PROGRESS` com fase `sandbox:...` (sem caminhos), para a auditoria do job (`ANALYSIS_COMPLETED.metadata.sandbox`) e, se alguma camada faltou, para o aviso `SANDBOX_NOT_FULLY_APPLIED` do job.

### D-6 — Ordem de custo (decisão do operador sobre o que fazer)

1. **Feito:** F1-22 (D-2).
2. **Feito:** Landlock + seccomp no Linux (D-3).
3. **Feito, não verificado em Windows:** Job Object (D-4). Token restrito/AppContainer: **não feito**.
4. **Sem código:** analisar em máquina ou VM dedicada e offline (spec §29.3), documentado em `docs/OPERATION.md`. Continua sendo a recomendação para evidência de origem desconhecida.

### D-7 — Exibição do log: caracteres de direção de texto

Texto de log mostrado ao analista ou exportado passa por `text_safety`: caracteres de direção (`U+202A..202E`, `U+2066..2069`, marcas LRM/RLM/ALM), de largura zero, BOM, hífen condicional e de etiqueta viram um marcador **visível** `⟦U+202E⟧`. O texto original continua intacto no banco e na evidência ("mostrar, não apagar"). Aplicado na timeline, nos candidatos de detecção e nos cinco documentos Markdown. Sem isso, `relatorio‮txt.exe` aparece como `relatorioexe.txt`. Risco original baixo (spoofing na tela); custo de corrigir, menor ainda.

## Consequências

- O README deixa de dizer "engine isolado" sem qualificar: no Linux, com `required`, o engine está **contido**; no Windows está **isolado por processo e limitado por Job Object**, sem confinamento de arquivo e rede.
- `RF-036` ("não existe fallback silencioso para execução sem sandbox") passa a ser literalmente verdadeiro: o fallback existe, é escolha explícita do operador e é registrado em cada job.
- **Em Linux aarch64 o padrão `required` faz o engine recusar analisar**, porque o seccomp só existe para x86_64 e a camada fica indisponível (falha fechada). Quem rodar em aarch64 precisa escolher `best-effort` conscientemente; o job registra que o seccomp faltou. Não foi testado em aarch64.
- A prova de contenção exige kernel com Landlock (≥ 5.13 com o LSM ativo). Em Linux sem ele, os testes de enforcement avisam e saem; o padrão `required` faz o engine recusar analisar até que o operador escolha `best-effort`.
- Mudanças em `Cargo.toml`: o engine ganha `libc` (Linux) e `windows-sys` (Windows). Ambos já estavam no grafo; o `Cargo.lock` ganha só as duas arestas.
