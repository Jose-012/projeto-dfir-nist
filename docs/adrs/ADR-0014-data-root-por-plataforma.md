<!-- doc-type: decision | status: accepted -->
# ADR-0014 — Default de `data_root` por plataforma (evita pastas sincronizadas na nuvem)

## Status

ACEITA — 2026-08-28

## Contexto

Levantada em conversa com o operador: onde o banco SQLite e o storage de evidências (`db/`, `evidence/`, `staging/`, `sanitized/`, `exports/`, `quarantine/`) ficam por padrão, quando `DFIR_DATA_ROOT` não é definido explicitamente.

O comportamento anterior (`default_data_root` em `src/config.rs`, desde a Fase 0): Windows já usava `%LOCALAPPDATA%\DFIR\storage` (estável, por usuário); Unix (Linux/macOS, tratados juntos) caía para `./storage`, **relativo ao diretório de trabalho no momento em que o binário é iniciado**.

Duas perguntas foram feitas nessa conversa: (1) o storage/binário do engine deveriam ficar dentro da pasta do projeto (código-fonte)? (2) há risco de exposição em usar uma pasta de "dados de aplicativo" do usuário?

## Decisão

### Não: dados de evidência não pertencem à árvore de código-fonte

A pasta do projeto é um repositório git. Evidência ali dentro é um `git add .`/`git clean -fdx`/push acidental de distância de virar um incidente de vazamento ou perda de cadeia de custódia — e numa instalação real (não um checkout de desenvolvimento) essa pasta nem existe, só o binário instalado. Isso não muda com esta ADR; já era a postura implícita do projeto (o storage sempre viveu fora da árvore versionada) e continua sendo.

### Sim, há uma exposição real — mas ela aponta para consertar o default `./storage`, não para evitar diretórios de dados de aplicativo

`./storage` sendo relativo ao CWD tem duas falhas práticas:

1. **Inconsistência**: iniciar o binário a partir de locais diferentes (atalho na Área de Trabalho, terminal, serviço) produz storages diferentes e desconectados entre si, sem aviso.
2. **Risco de sincronização em nuvem**: se o CWD no momento da inicialização for a Área de Trabalho ou Documentos — justamente as pastas que o OneDrive "Known Folder Move" e o iCloud Drive costumam redirecionar/sincronizar por padrão em muitos ambientes — `./storage` colocaria banco e evidência ali dentro, sincronizando tudo silenciosamente para a nuvem.

A pergunta do operador foi, em essência, "usar uma pasta de dados do usuário não seria um risco parecido?" — não, é o oposto: `%LOCALAPPDATA%` (Windows), `~/Library/Application Support` (macOS) e `$XDG_DATA_HOME`/`~/.local/share` (Linux) são especificamente as pastas que essas mesmas ferramentas de sincronização **não** redirecionam por padrão (o OneDrive KFM cobre Área de Trabalho/Documentos/Imagens; o iCloud Drive cobre Área de Trabalho/Documentos quando habilitado — nenhum dos dois cobre AppData/Application Support por padrão). Trocar `./storage` por essas pastas reduz o risco que a pergunta levantou, não o introduz.

### Implementação

`src/config.rs`: `default_data_root()` passa a despachar por uma enum explícita `HostPlatform` (Windows/MacOs/OtherUnix) para uma função pura `select_data_root`, que recebe os valores de `LOCALAPPDATA`/`XDG_DATA_HOME`/`HOME` como parâmetros (não lidos diretamente) — isso permite testar os quatro ramos (Windows, macOS, Linux com `XDG_DATA_HOME`, Linux caindo para `HOME`) e o fallback final a partir de qualquer host de CI, com valores sintéticos, sem depender de qual plataforma realmente compila os testes nem mutar env vars reais do processo (mesma razão de design já documentada para `windows_data_root` antes desta ADR).

Resultado por plataforma quando `DFIR_DATA_ROOT` não é definido:
- Windows: `%LOCALAPPDATA%\DFIR\storage` (inalterado)
- macOS: `~/Library/Application Support/DFIR/storage` (novo — antes caía no `./storage` genérico de Unix)
- Linux: `$XDG_DATA_HOME/dfir/storage` se definido, senão `~/.local/share/dfir/storage` (novo)
- Fallback final (nenhuma variável de ambiente relevante presente, em qualquer plataforma): `./storage`, preservando o comportamento antigo nesse caso extremo

`src/main.rs`: o `data_root` resolvido passa a ser logado (`tracing::info!`) na inicialização — para que o analista sempre veja, no próprio log estruturado, para onde a evidência está indo, em vez de esse caminho ficar implícito.

## Alternativas consideradas

- **Manter `./storage` no Unix.** Rejeitada pela própria pergunta que originou esta ADR — o risco de sincronização em nuvem via CWD é real e específico deste tipo de ferramenta.
- **Adicionar a crate `dirs` para resolver os diretórios convencionais.** Rejeitada por "necessidade demonstrada": a lógica de junção de caminho para as três plataformas é trivial (poucas linhas cada) e já seguia esse padrão manual para o Windows desde a Fase 0; adicionar uma dependência nova para evitar repetir esse padrão não se justifica.
- **Exigir `DFIR_DATA_ROOT` explícito sempre, sem nenhum default.** Rejeitada: um default seguro e prático continua sendo melhor UX do que forçar configuração manual antes do primeiro uso; quem quer um diretório por caso (o padrão de trabalho recomendado para uma investigação) continua podendo usar `DFIR_DATA_ROOT` normalmente.

## Consequências

Positivas:
- Default consistente e seguro nas três plataformas, sem depender de onde o binário é iniciado.
- Cobertura de teste estritamente melhor que antes: os quatro ramos de seleção agora têm teste determinístico próprio, em vez de um único teste que só exercitava o ramo da plataforma que rodava a suíte.
- Log de inicialização torna o destino da evidência visível, não implícito.

Negativas / mudança de comportamento:
- Quem já usava o default antigo no macOS (`./storage`) verá o caminho mudar para `~/Library/Application Support/DFIR/storage` na próxima execução sem `DFIR_DATA_ROOT` definido — dados existentes em `./storage` não são movidos automaticamente. Como o projeto ainda não teve nenhuma instalação real em campo (Fase 1 em desenvolvimento), o impacto prático é nulo agora, mas fica registrado aqui caso surja depois.

## Evidência de execução

Nenhum gate Rust executado neste ambiente (sem `cargo`/`rustc`). Testes novos/atualizados em `src/config.rs`: um teste unitário por função pura (`windows_data_root`, `macos_data_root`, `linux_xdg_data_root`, `linux_home_data_root`), um teste de despacho cobrindo os quatro casos de `select_data_root` com valores sintéticos (incluindo a precedência de `XDG_DATA_HOME` sobre `HOME`), um teste de fallback para as três plataformas quando nenhuma env var está presente, e um smoke test de `default_data_root()` no host real (não determinístico por design, já que lê env vars reais). O teste antigo (`default_data_root_is_relative_storage_off_windows`), cuja premissa não vale mais, foi substituído pelos acima. Revisado manualmente linha a linha, mas não compilado.

## Adendo — 2026-09-21 — portabilidade dos testes

A afirmação de que os quatro ramos são testáveis "a partir de qualquer host" era falsa para as asserções do Windows: `PathBuf::join` usa o separador do host, então a string literal `C:\Users\…\DFIR\storage` só coincide no Windows (em Linux, `\` não é separador e o resultado leva `/`). Os dois testes afetados agora comparam a estrutura (`base` + `DFIR` + `storage`, montada com o próprio `join`) em qualquer host e mantêm a asserção literal sob `#[cfg(windows)]`. Nenhuma mudança no código de produção.
