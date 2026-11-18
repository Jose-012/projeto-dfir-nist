<!-- doc-type: source | status: current -->
# Demanda original — SAGA SENAI de Inovação, demanda 12412

> **Fonte primária de requisitos de negócio** (ver `docs/DOCUMENT_AUTHORITY.md`, linha "Demanda original"). Rastreabilidade item a item: `docs/DEMAND_TRACE.md`.
> Transcrição do texto da página da plataforma, impressa em 23/04/2026 18:04.
> URL de origem: https://plataforma.gpinovacao.senai.br/plataforma/demandas-da-industria/interna/12412
>
> **Regras deste arquivo:** não editar o texto transcrito. Qualquer interpretação, decisão ou desvio mora na especificação, nunca aqui.
> **Cópia literal:** o texto da impressão de 23/04/2026 fornecido pelo operador, sem edição (inclui o cabeçalho da plataforma e ideias de outras equipes), está em `docs/source/demanda-12412-impressao-2026-04-23.txt` (SHA-256 abaixo). Este arquivo `.md` é a transcrição limpa e é o que `docs/DEMAND_TRACE.md` referencia.
>
> **PENDING-HUMAN:** guardar o PDF original da impressão ao lado, como `docs/source/demanda-12412.pdf`, e registrar seu SHA-256 abaixo (o PDF não estava disponível; só o texto).
>
> SHA-256 de `demanda-12412-impressao-2026-04-23.txt`: `52bd7792514dd1989c741ca54b1e691fb18f6b342e53136a9dda1f0339f49b41`
> SHA-256 do PDF original: `(preencher após salvar o arquivo)`

## Identificação

- **Programa:** Projeto Integrador 2026/01
- **Título:** 1.32 01: Avaliação Forense de Incidente Cibernético e Fortalecimento da Resiliência com Base no NIST CSF 2.0
- **Empresa:** Avivatec Soluções em Tecnologia da Informação Ltda.
- **Escola:** (SP) 1.32 Escola SENAI de Informática
- **Estado:** SP
- **Área de atuação:** Tecnologia da Informação
- **Data de cadastro:** 04/03/2026 21:26
- **Vigência:** 04/03/2026 a 04/03/2027 (12 meses)
- **Ativo:** Sim

## Descrição resumida

A organização enfrenta incidentes cibernéticos cada vez mais sofisticados que podem comprometer sistemas críticos, dados sensíveis e a continuidade operacional. A falta de um processo estruturado de investigação dificulta identificar a causa raiz e prevenir novos ataques. Propõe-se realizar uma avaliação baseada no NIST CSF 2.0, com análise forense das evidências, identificação de vulnerabilidades e elaboração de um plano de hardening e criptografia para aumentar a resiliência cibernética.

## Benefícios esperados

1. Identificar a causa raiz do incidente cibernético ocorrido.
2. Melhorar a capacidade de resposta a incidentes de segurança.
3. Reduzir vulnerabilidades existentes na infraestrutura de TI.
4. Aumentar a resiliência cibernética da organização.
5. Fortalecer a proteção de dados sensíveis e informações críticas.
6. Reduzir riscos de novos ataques cibernéticos.
7. Melhorar o monitoramento e detecção de ameaças.
8. Implementar práticas adequadas de hardening de sistemas.
9. Implantar criptografia adequada para proteção de dados em trânsito e em repouso.
10. Alinhar a organização às boas práticas internacionais de segurança da informação (NIST).

## Detalhamento

A demanda consiste na realização de uma avaliação técnica e forense de um incidente cibernético, seguida da elaboração de um plano estruturado de melhoria da segurança da informação. Inicialmente será conduzido um processo de investigação forense digital, envolvendo a coleta, preservação e análise de evidências digitais provenientes de diferentes fontes da infraestrutura tecnológica, como: logs de servidores, registros de autenticação, tráfego de rede, sistemas de monitoramento, endpoints e estações de trabalho, dispositivos de segurança.

Durante essa etapa serão utilizadas ferramentas de análise forense e monitoramento, como plataformas de SIEM, análise de logs, análise de tráfego de rede e investigação de artefatos digitais, com o objetivo de identificar: vetor inicial do ataque, vulnerabilidades exploradas, movimentação lateral dentro da rede, escalonamento de privilégios, possíveis tentativas de exfiltração de dados, impacto sobre sistemas e serviços. Com base nas evidências coletadas, será construída uma linha do tempo do incidente, permitindo compreender de forma detalhada o comportamento do atacante e os mecanismos utilizados para comprometer a infraestrutura.

Após a investigação forense, será realizada uma avaliação da postura de segurança da organização com base no NIST Cybersecurity Framework 2.0, analisando aspectos relacionados à governança, identificação de ativos, controles de proteção, monitoramento, resposta a incidentes e capacidade de recuperação.

A partir dessa análise será desenvolvido um plano de fortalecimento da segurança, incluindo:

- implementação de políticas de hardening de sistemas operacionais e servidores.
- revisão de configurações de segurança em redes e dispositivos.
- melhoria de mecanismos de autenticação e controle de acesso.
- implantação ou aprimoramento de soluções de monitoramento de segurança.
- implementação de criptografia adequada para dados armazenados e dados em trânsito.
- revisão da gestão de chaves criptográficas.
- definição de políticas e procedimentos de resposta a incidentes

O resultado final do projeto será um conjunto de entregáveis técnicos, incluindo:

- relatório completo de análise forense do incidente
- identificação das vulnerabilidades exploradas
- avaliação de maturidade em segurança baseada no NIST CSF 2.0
- recomendações de melhoria da postura de segurança
- plano de hardening e fortalecimento da infraestrutura
- plano de implementação de criptografia e proteção de dados

Essas ações contribuirão diretamente para o aumento da resiliência cibernética da organização, reduzindo a probabilidade de novos incidentes e aumentando a capacidade de resposta a ameaças.

## Restrições (o que a solução não pode ter)

1. Disponibilidade limitada de logs ou evidências digitais devido a retenção insuficiente de registros
2. Dependência da colaboração de diferentes áreas técnicas da organização para levantamento de informações
3. Possível indisponibilidade temporária de sistemas durante análises ou testes de segurança
4. Limitações de orçamento para implementação imediata de todas as recomendações de segurança
5. Necessidade de alinhamento com políticas internas e requisitos regulatórios da organização
6. Restrições relacionadas à confidencialidade de dados e privacidade durante o processo de investigação

## Ideias registradas na demanda (relevantes a este projeto)

- **310326** — "Comprometimento de Credenciais e Fraude Financeira via Phishing (Business Email Compromise)". Desafio: UniSENAI SP 2026/01 — Projeto Integrador. Cadastro: 17/04/2026.
- **310327** — mesmo título (registro adicional). Cadastro: 17/04/2026.

> As demais ideias listadas na página pertencem a outras equipes e não fazem parte do escopo deste projeto.
