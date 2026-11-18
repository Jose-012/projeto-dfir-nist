<!-- doc-type: decision | status: accepted -->
# ADR-0002 — Rejeição de Hayabusa e Chainsaw como componentes do produto

**Status:** ACEITA / REJEITADO PARA INCORPORAÇÃO

**Data:** 2026-08-17

## Contexto

Foram avaliadas as ferramentas Hayabusa e Chainsaw como possíveis simplificações do pipeline de análise de EVTX/Sigma. A proposta foi rejeitada para incorporação ao produto porque o projeto deve preservar a possibilidade de distribuição comercial/licenciamento próprio, além de manter parser/detecção próprios e uma cadeia de supply-chain controlada.

## Evidência de licença

A documentação/repositório oficial atualmente declara Hayabusa sob **GNU AGPLv3** e suas regras sob **Detection Rule License (DRL) 1.1**. O repositório oficial de Chainsaw declara **GPL-3.0**. Essas informações são evidência técnica de licenciamento dos artefatos pesquisados, não parecer jurídico.

## Decisão

**Hayabusa e Chainsaw não fazem parte do produto.** Não serão adicionados como:

- dependência Cargo;
- binário distribuído;
- submódulo/vendor;
- ruleset incorporado;
- componente de container/Quadlet;
- dependência de runtime;
- fixture necessária para operação.

A Fase 0 permanece exclusivamente com os componentes definidos no documento de arquitetura. A futura análise EVTX/detecção continuará usando a arquitetura Rust própria e suas dependências explicitamente aprovadas, sujeitas aos gates de necessidade, licença, segurança e desempenho.

## Razões

1. preservação da opção de distribuição comercial e de licenciamento próprio;
2. redução da superfície de supply-chain e de atualização;
3. preservação da curadoria/proveniência do ruleset do projeto;
4. prevenção de dependência arquitetural em uma suíte DFIR externa;
5. aderência ao Princípio de Necessidade Demonstrada.

## O que não significa

Esta ADR não afirma que chamar qualquer ferramenta externa por subprocesso é automaticamente uma violação jurídica. A conclusão jurídica depende do uso, modificações, agregação e modelo efetivo de distribuição e deve ser avaliada por autoridade jurídica competente antes de distribuição. A decisão aqui é deliberadamente mais conservadora: essas duas ferramentas não serão componentes do produto.

## Consequências

- não precisamos acomodar Hayabusa/Chainsaw no processo separado do produto;
- não precisamos incluir suas regras no inventário de componentes do produto;
- a integração futura de outra ferramenta DFIR exige nova ADR;
- qualquer reavaliação deve considerar licença do executável, regras/dados, dependências, atualizações, proveniência e distribuição;
- a equipe mantém responsabilidade sobre o parser/detecção adotados pelo próprio projeto.

## Fontes externas consultadas

- Hayabusa: repositório oficial e README, que declara AGPLv3 para o software e DRL 1.1 para regras.
- Hayabusa Rules: repositório oficial, que declara DRL 1.1.
- Chainsaw: repositório oficial, que declara GPL-3.0.
