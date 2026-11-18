<!-- doc-type: decision | status: accepted -->
# ADR-0015 — `engine_path` resolvido a partir do executável do `dfir-app`

## Status

ACEITA — 2026-09-21

## Contexto

O app localiza o binário `dfir-engine` por `AppConfig::engine_path`. Antes desta ADR o padrão era:

- Windows: `<data_root>/runtime/dfir-engine.exe` — um diretório dentro do storage de **dados** do usuário (`%LOCALAPPDATA%\DFIR\storage\runtime\`), populado pelo `scripts/setup.ps1`;
- Unix: `/usr/local/bin/dfir-engine` — uma instalação de sistema, que exige privilégio de administrador.

Isso destoa do que a ferramenta se propõe a ser (local, sem administrador, autocontida): código executável misturado com dados de evidência no Windows, e uma exigência de instalação de sistema no Unix. Também tornava o `just` de desenvolvimento dependente de um passo de "instalação" que copia o engine recém-compilado para outro lugar.

## Decisão

`default_engine_path` resolve, nesta ordem (a primeira que existir vence):

1. **`DFIR_ENGINE_PATH`** — sobrescrita explícita do operador, respeitada quando não vazia, mesmo que o arquivo não exista (o erro aparece ao iniciar o job, apontando o caminho pedido). Um valor vazio é tratado como não definido.
2. **`<diretório de current_exe()>/dfir-engine[.exe]`**, se for um arquivo. `cargo build --workspace` já gera `dfir-app` e `dfir-engine` lado a lado em `target/<perfil>/`, e uma distribuição ("pasta com dois executáveis") mantém o mesmo layout — não há passo de instalação.
3. **Alternativa por plataforma**: no Unix, `/usr/local/bin/dfir-engine` (mantém o comportamento anterior para quem instalou o engine no sistema); no Windows não há local de sistema equivalente, e o caminho do passo 2 é devolvido mesmo sem existir, para que a mensagem de erro mostre onde o engine foi procurado. Se nem o diretório do executável puder ser determinado, o Windows cai no local legado `<data_root>/runtime/dfir-engine.exe` — nunca em um caminho relativo ao diretório de trabalho.

A seleção é uma função pura (`select_engine_path`), no mesmo molde da ADR-0014: recebe a sobrescrita, o diretório do executável, a plataforma e um predicado de existência como parâmetros, e por isso cada ramo é testável de qualquer host sem tocar no sistema de arquivos nem no ambiente reais. `src/main.rs` registra no log o caminho resolvido e se o arquivo existe, para que ele nunca fique implícito.

`current_exe()` no Linux resolve o link simbólico de `/proc/self/exe`: se o `dfir-app` for acessado por um link, o engine é procurado ao lado do executável **real**.

## Consequências

Positivas:

- Funciona sem administrador e sem passo de instalação: rodar de `target/debug` ou de uma pasta descompactada encontra o engine.
- O storage de dados deixa de receber uma cópia do binário do engine no Windows: `scripts/setup.ps1` não copia mais o `dfir-engine.exe` para `storage\\runtime\\`. (O diretório `runtime/` continua fazendo parte do layout do storage criado pelo app; só o engine deixa de ser colocado nele.)
- Desenvolvimento e distribuição usam o mesmo layout.

Negativas / mudança de comportamento:

- Quem instalou o engine em `%LOCALAPPDATA%\DFIR\storage\runtime\dfir-engine.exe` (padrão antigo do Windows) não o terá mais encontrado automaticamente: mova-o para o lado do `dfir-app.exe` ou defina `DFIR_ENGINE_PATH`. Como ainda não houve instalação em campo, o impacto prático é nulo, mas fica registrado.
- Testes e scripts que executam o app a partir de um caminho que não seja o do `cargo build` (por exemplo, `target/debug/deps/…` dentro de `cargo test`) devem definir `DFIR_ENGINE_PATH`, como já fazem `tests/integration/fase0_runtime.*`.
- Se o atacante controlar o diretório do executável, ele já controla o próprio `dfir-app`; resolver o engine ali não amplia a superfície. O caminho continua sendo canonicalizado antes do `spawn` (`process.rs`).

## Alternativas consideradas

- **Manter `/usr/local/bin` como padrão principal no Unix.** Rejeitada: pressupõe privilégio de administrador e um passo de instalação.
- **Embutir o engine no binário do app e extraí-lo em tempo de execução.** Rejeitada: aumentaria o app com o grafo pesado do `yara-x`/`wasmtime`, e extrair um executável para o disco a cada início é uma superfície pior que resolver um irmão no mesmo diretório.
- **Um único binário com subcomandos.** Rejeitada: a separação em processo distinto é uma decisão de isolamento do parser (ADR-0006) — o engine ter seu próprio executável é o que permite limitá-lo (`RLIMIT_AS`, timeout) sem afetar o app.

## Evidência de execução

Testes em `src/config.rs` (ramos de `select_engine_path` para as três plataformas). Verificado ainda de ponta a ponta: `dfir-app` iniciado **sem** `DFIR_ENGINE_PATH`, a partir de `target/debug`, executou um `YARA_SCAN` real pelo `dfir-engine` vizinho.
