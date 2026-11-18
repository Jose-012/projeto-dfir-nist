<!-- doc-type: decision | status: accepted -->
# ADR-0028 — SBOM fora do escopo

## Status

ACEITA — 2026-11-09 (decisão do operador, em resposta à auditoria documental de 2026-11-04).

## Contexto

A especificação 6.4.1 pedia SBOM em quatro pontos: `RF-024` ("SBOM/licenças — Documentação gerada com versões reais"), `RF-031` ("Release possui `Cargo.lock`, SBOM e imagens identificadas por digest"), a seção 41.1 (artefatos a gerar) e o item 2 da F.12 ("dependências, lockfile e SBOM reconciliados"). Esses trechos, e o título da própria seção 41, desapareceram da especificação na 6.4.5 sem ADR nem registro, e `docs/FASE0_STATUS.md` ainda listava `cyclonedx` como gate pendente.

A auditoria de 2026-11-04 (gaps AUS-02, INC-16 e VAZ-01) perguntou ao operador se o corte foi decisão ou regressão de edição. Resposta do operador: "SBOM permanece não-presente. Não é um projeto corporativo."

## Decisão

1. **SBOM (CycloneDX ou SPDX) está fora do escopo.** Não há `SBOM.json`, receita `just` nem gate de SBOM; `cyclonedx` deixa de ser gate pendente.
2. **`RF-024` e `RF-031` voltam à tabela da seção 39, reescritos** (especificação 6.5.1, seção 39.2), com os IDs preservados: `RF-024` passa a exigir `LICENSE` e `LICENSES_THIRD_PARTY.md` com as versões reais do `Cargo.lock`; `RF-031` passa a exigir o `Cargo.lock` e a execução registrada de `cargo audit` e `cargo deny --workspace check`.
3. **O inventário de licenças continua obrigatório** e vem de `cargo metadata --locked --format-version 1` (executado em 2026-11-09: 672 pacotes, todos com licença declarada).
4. **A seção 41, a F.8.1 e o item 2 da F.12 são restaurados sem SBOM e sem imagens.**

## Alternativas rejeitadas

- **Restaurar o SBOM da 6.4.1:** contraria a decisão do operador; o projeto não é distribuído de forma corporativa.
- **Manter a omissão silenciosa:** viola o histórico append-only e deixa IDs normativos (`RF-024`, `RF-031`) citados sem existir na tabela.

## Consequências

- A rastreabilidade de dependências é o `Cargo.lock`, o `LICENSES_THIRD_PARTY.md` e os gates `cargo audit` e `cargo deny --workspace check`.
- Se o projeto passar a ser distribuído a terceiros, esta decisão deve ser revista por uma nova ADR que a `supersedes`.
- A validação jurídica antes de distribuir (especificação, seção 41.3) permanece.

## Supersedes

Parcialmente, o texto de `RF-024`, `RF-031` e da seção 41.1 da especificação 6.4.1 (não havia ADR anterior sobre SBOM).

## Superseded-by

Nenhum.
