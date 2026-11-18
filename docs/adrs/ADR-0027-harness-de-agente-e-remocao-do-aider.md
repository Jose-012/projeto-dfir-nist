<!-- doc-type: decision | status: accepted -->
# ADR-0027 — Harness de agente e remoção do Aider

## Status

ACEITA — 2026-11-04 (decisão do operador).

## Contexto

O Aider era a ferramenta de desenvolvimento assistido do projeto. O `.aider.conf.yml` carregava `AGENTS.md`, `CONVENTIONS.md` e `docs/DOCUMENT_AUTHORITY.md` em toda sessão, desligava o commit automático e, junto com o `.aiderignore`, mantinha fora do contexto evidências, bancos, secrets e artefatos gerados. O `CONVENTIONS.md` era escrito "para Aider". Em 2026-11-04 o operador decidiu substituir o Aider por um harness de agente.

## Decisão

1. **Remover o Aider por completo.** Apagar `.aider.conf.yml` e `.aiderignore` (o diretório `.aider/` já estava ausente) e retirar as referências a ele dos documentos vivos. A entrada histórica de `CHANGELOG.md` permanece, marcada "Removido em 2026-11 por ADR-0027".
2. **Tornar `CONVENTIONS.md` agnóstico de harness** ("Convenções de desenvolvimento para agentes LLM"). As regras úteis dos arquivos removidos migram para ele: não commitar automaticamente, não ampliar o escopo em silêncio para arquivos ignorados pelo Git e manter evidências, bancos, secrets e artefatos gerados fora do contexto automático do agente.
3. **Não nomear o harness nos documentos vivos.** O harness usado no desenvolvimento é o **Jcode** (declaração do operador em 2026-11-04). O nome fica registrado só nesta ADR e em `docs/TOOLCHAINS.md`, como ferramenta que foi usada. ~~Versão, licença, repositório e supply chain do Jcode estão `[PENDING-HUMAN]` e precisam ser resolvidos antes de qualquer gate que dependa dele; hoje nenhum depende.~~ Substituído em 2026-11 pela atualização abaixo: o Jcode não é ferramenta obrigatória.

~~`[PENDING-HUMAN]` — pergunta objetiva: o harness atual impede o commit automático e a gravação do histórico de conversa no repositório, como o Aider fazia por configuração?~~ Respondida pelo operador em 2026-11 (atualização abaixo).

## Alternativas rejeitadas

- **Cravar o Jcode em `CONVENTIONS.md`:** acopla o conteúdo à ferramenta e obriga a reescrever o arquivo a cada troca de harness.
- **Manter o Aider:** contraria a decisão do operador.
- **Trocar de harness sem ADR:** viola `AGENTS.md` §9, item 6 (atualizar ADR se a decisão mudar).

## Consequências

- O conteúdo de `CONVENTIONS.md` sobrevive a uma troca futura de harness.
- A escolha do harness passa a ser uma decisão registrada; o operador pode trocar de harness com uma nova ADR que `supersedes` esta.
- A configuração do Aider que não migrou (histórico local, verificação de atualização, TLS) saiu com a ferramenta. ~~O equivalente no harness atual depende da pergunta `[PENDING-HUMAN]` acima.~~ Ver a atualização abaixo.
- Nenhum gate, receita `just`, `deny.toml` ou CI depende do harness.
- Estado dos artefatos na remoção: `.aider.conf.yml` e `.aiderignore` apagados; `.aider/` já ausente.

## Supersedes

Nenhum.

## Superseded-by

Nenhum.

## Atualização — 2026-11 (resposta do operador)

- O Jcode consta nos documentos apenas como a ferramenta que foi usada. Não é ferramenta obrigatória: nenhum gate, receita `just`, `deny.toml`, CI ou documento depende dele, e não há requisito de versão, licença, repositório ou supply chain para ele.
- O harness usado **não impede** o commit automático nem a gravação do histórico de conversa. Nada na configuração do harness reforça as regras migradas para `CONVENTIONS.md` (não commitar sem pedido, não ampliar o escopo para arquivos ignorados pelo Git, manter evidências fora do contexto): elas dependem do agente e da revisão do operador.
