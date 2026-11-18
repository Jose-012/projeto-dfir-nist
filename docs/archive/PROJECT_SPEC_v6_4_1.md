# DOCUMENTO DE ARQUITETURA, ESPECIFICAÇÃO TÉCNICA E DIRETRIZES DE IMPLEMENTAÇÃO

## Versão 6.4.1 — Especificação definitiva para implementação controlada, com governança operacional, UX, escopo explícito de parsing e catálogo de evolução

```xml
<documento-projeto>
  <identificacao>
    <versao>6.4.1</versao>
    <status>ESPECIFICAÇÃO REFINADA — ARQUITETURA FECHADA E BASE CONTROLADA DE IMPLEMENTAÇÃO</status>
    <data>2026-08-17</data>
    <programa>SAGA SENAI de Inovação 2026/01 — Projeto Integrador</programa>
    <escola>(SP) 1.32 Escola SENAI de Informática</escola>
    <empresa-parceira>Avivatec Soluções em Tecnologia da Informação Ltda.</empresa-parceira>
    <codigo-demanda>12412</codigo-demanda>
    <titulo>Avaliação Forense de Incidente Cibernético e Fortalecimento da Resiliência com Base no NIST CSF 2.0</titulo>
    <publico-alvo>
      Desenvolvedores, Engenheiros de Segurança, Modelos de Linguagem (LLMs),
      Avaliadores da Banca SENAI e Equipe de Qualidade
    </publico-alvo>
  </identificacao>

  <escopo>
    Plataforma local de DFIR para coleta/importação, preservação, análise e correlação
    de evidências digitais; avaliação da postura de segurança com NIST CSF 2.0;
    identificação de vulnerabilidades e caminhos do incidente; elaboração de
    recomendações, hardening, criptografia e procedimentos de resposta; geração
    rastreável de entregáveis técnicos.
  </escopo>

  <fonte-da-demanda status="fornecida-pelo-usuario">
    A fonte primária dos requisitos de negócio é a demanda SAGA SENAI 12412 fornecida
    pelo usuário nesta conversa. Decisões arquiteturais desta especificação não devem
    ser confundidas com exigências originais do SENAI/Avivatec.
  </fonte-da-demanda>

  <regra-principal>
    Nenhuma LLM pode transformar hipótese, estimativa, exemplo ou decisão de engenharia
    em requisito de negócio sem registrar a alteração e sua justificativa.
  </regra-principal>

  <estado-legal>
    <inventario-de-licencas>REALIZADO COMO ANALISE TECNICA, NAO COMO PARECER JURIDICO</inventario-de-licencas>
    <comercializacao>SUJEITA A VALIDACAO JURIDICA DO MODELO EFETIVO DE DISTRIBUICAO</comercializacao>
    <velociraptor>OPCIONAL, FASE 4, SEM MODIFICACAO DO COMPONENTE NO PROJETO</velociraptor>
  </estado-legal>
</documento-projeto>
```

> **Princípio de governança:** este documento é uma especificação de engenharia. O inventário de licenças serve para orientar implementação e distribuição; não constitui parecer jurídico.

---

# 1. Convenções normativas
<!-- @section id="1" slug="convencoes-normativas" -->

| Termo | Significado |
|---|---|
| **OBRIGATÓRIO** | Deve existir na implementação correspondente. |
| **RECOMENDADO** | Deve ser implementado salvo justificativa técnica documentada. |
| **OPCIONAL** | Não bloqueia o MVP. |
| **HIPÓTESE** | Ainda precisa de benchmark, fonte primária ou teste. |
| **EXEMPLO** | Ilustrativo; não significa código de produção. |
| **DECISÃO** | Escolha arquitetural desta versão. |
| **PENDENTE** | Depende de informação externa à especificação ou decisão humana. |
| **CRITÉRIO DE ACEITAÇÃO** | Condição objetiva para considerar uma funcionalidade concluída. |
| **FONTE** | Requisito ou informação diretamente derivada da demanda original. |
| **REJEITADO** | Sugestão avaliada e não incorporada por incompatibilidade, risco ou complexidade desnecessária; deve possuir justificativa. |

## 1.1. Hierarquia de precedência

Em caso de conflito:

1. requisitos explícitos da demanda SAGA SENAI 12412;
2. requisitos consolidados nesta especificação;
3. decisões arquiteturais desta versão;
4. hipóteses, exemplos e sugestões.

Uma decisão arquitetural nunca pode apagar um requisito explícito da demanda.

### 1.2. Camadas de representação e autoridade

O documento utiliza Markdown como formato canônico para leitura humana e preservação integral do conteúdo normativo. Blocos XML presentes no documento são representações estruturadas derivadas para indexação, consumo por LLMs, rastreabilidade e automação; **não são uma segunda fonte de verdade**.

A autoridade segue esta ordem: (1) requisitos explícitos da demanda; (2) corpo normativo desta especificação; (3) decisões e critérios identificados por IDs estáveis; (4) índices/XML derivados; (5) exemplos, hipóteses e texto auxiliar. Se uma representação estruturada divergir do corpo normativo, o corpo prevalece e a representação deve ser corrigida/regenerada.

A LLM deve distinguir pelo menos `requirement`, `decision`, `constraint`, `acceptance-criterion`, `procedure`, `llm-rule`, `gate`, `hypothesis`, `pending` e `rejected`. Esses rótulos descrevem a natureza da informação; não elevam automaticamente um texto a requisito de negócio.

---

# 2. Demanda original consolidada
<!-- @section id="2" slug="demanda-original-consolidada" -->

## 2.1. Requisitos confirmados

A demanda original solicita uma avaliação técnica e forense de um incidente cibernético seguida de um plano estruturado de melhoria da segurança da informação.

O escopo confirmado inclui:

- coleta, preservação e análise de evidências digitais;
- logs de servidores;
- registros de autenticação;
- tráfego de rede;
- sistemas de monitoramento;
- endpoints e estações de trabalho;
- dispositivos de segurança;
- identificação do vetor inicial;
- identificação de vulnerabilidades exploradas;
- investigação de movimentação lateral;
- investigação de escalonamento de privilégios;
- investigação de possíveis tentativas de exfiltração;
- avaliação do impacto em sistemas e serviços;
- construção da linha do tempo;
- avaliação da postura de segurança com NIST CSF 2.0;
- hardening de sistemas operacionais e servidores;
- revisão de configurações de redes e dispositivos;
- melhoria de autenticação e controle de acesso;
- implantação/aprimoramento de monitoramento;
- criptografia em repouso e em trânsito;
- revisão de gestão de chaves;
- políticas e procedimentos de resposta a incidentes.

## 2.2. Benefícios esperados pela demanda

- identificar causa-raiz;
- melhorar resposta a incidentes;
- reduzir vulnerabilidades;
- aumentar resiliência cibernética;
- proteger dados sensíveis e informações críticas;
- reduzir risco de novos ataques;
- melhorar monitoramento e detecção;
- fortalecer hardening;
- implementar criptografia adequada;
- alinhar-se ao NIST.

## 2.3. Contexto prioritário BEC

A demanda está associada ao desafio de **Comprometimento de Credenciais e Fraude Financeira via Phishing (Business Email Compromise — BEC)**.

**DECISÃO:** o cenário BEC será o principal cenário do dataset de demonstração, sem limitar a plataforma a BEC.

O dataset e as heurísticas iniciais podem priorizar:

- autenticações anômalas;
- acesso via e-mail/serviços de identidade;
- regras de encaminhamento suspeitas;
- alterações de credenciais;
- origem geográfica/comportamental incomum;
- acesso a dados financeiros;
- PowerShell ou outras etapas pós-comprometimento, quando presentes no dataset.

---

# 3. Restrições da demanda
<!-- @section id="3" slug="restricoes-da-demanda" -->

| ID | Restrição | Tratamento arquitetural |
|---|---|---|
| RB-001 | Retenção insuficiente de logs/evidências | Trabalhar com múltiplas fontes e declarar lacunas explicitamente. |
| RB-002 | Dependência de diferentes áreas | Fluxos assíncronos e coleta automatizável quando autorizada. |
| RB-003 | Possível indisponibilidade | Análise offline e coleta minimamente intrusiva. |
| RB-004 | Limitação orçamentária | Prioridade para componentes open source e ausência de dependência obrigatória de serviços pagos. |
| RB-005 | Políticas/regulação | Governança, trilhas de auditoria e separação de dados. |
| RB-006 | Confidencialidade/privacidade | Execução local, least privilege, dados sanitizados e modo offline. |

> **Importante:** restrição não significa problema já solucionado. A implementação deve produzir evidência de que cada resposta arquitetural realmente funciona.

---

# 4. Entregáveis e rastreabilidade
<!-- @section id="4" slug="entregaveis-e-rastreabilidade" -->

A demanda original contém **seis requisitos de entrega/conteúdo**. A especificação propõe empacotá-los em cinco documentos formais para simplificar a apresentação, sem reduzir o conteúdo exigido. A divisão em cinco documentos é uma decisão de apresentação/engenharia, não uma alteração da demanda.

## 4.1. Requisitos de entrega originais

1. Relatório completo de análise forense do incidente.
2. Identificação das vulnerabilidades exploradas.
3. Avaliação de maturidade em segurança baseada no NIST CSF 2.0.
4. Recomendações de melhoria da postura de segurança.
5. Plano de hardening e fortalecimento da infraestrutura.
6. Plano de implementação de criptografia e proteção de dados.

## 4.2. Empacotamento proposto

### Entregável 1 — Relatório DFIR

Cobre principalmente o requisito 1 e a análise de causa-raiz **quando sustentada pelas evidências**. Quando as evidências não permitirem determinar a causa-raiz com segurança, o relatório deve registrar hipótese de causa provável, fatores contribuintes e lacunas, sem apresentar inferência como fato.

Inclui:

- resumo executivo;
- escopo e limitações;
- cadeia de custódia;
- evidências;
- timeline;
- vetor inicial;
- caminho crítico do incidente;
- movimentação lateral;
- escalonamento de privilégios;
- possíveis exfiltrações;
- impacto;
- causa-raiz, causa provável ou fatores contribuintes, conforme o nível de evidência;
- confiança e lacunas da conclusão;
- conclusões.

### Entregável 2 — Vulnerabilidades e MITRE ATT&CK

Cobre o requisito 2.

Inclui:

- vulnerabilidade/condição explorada;
- evidência;
- impacto;
- severidade;
- confiança;
- ATT&CK tactic/technique/sub-technique quando justificado;
- matriz de risco.

### Entregável 3 — NIST CSF 2.0 e Recomendações

Cobre os requisitos 3 e 4.

Inclui:

- Current Profile;
- Target Profile;
- Gap Analysis;
- prioridades;
- Tiers contextualizados;
- recomendações imediatas;
- recomendações estratégicas.

### Entregável 4 — Hardening

Cobre o requisito 5.

Inclui:

- achado;
- risco;
- recomendação;
- referência CIS quando aplicável;
- pré-condições;
- pós-condições;
- dry-run/check;
- playbook Ansible quando apropriado.

### Entregável 5 — Criptografia e Proteção de Dados

Cobre o requisito 6.

Inclui:

- classificação dos dados;
- proteção em trânsito;
- proteção em repouso;
- gestão de chaves;
- rotação;
- recuperação;
- matriz de proteção;
- plano de implementação.

## 4.3. Cobertura explícita das fontes citadas na demanda

A demanda original cita diferentes fontes de evidência. Isso não significa que o MVP precise possuir um conector ao vivo para cada produto de origem. Para evitar uma falsa impressão de cobertura, a especificação distingue **ingestão/preservação** de **análise especializada**:

| Fonte citada na demanda | MVP | Cobertura posterior |
|---|---|---|
| Logs de servidores | Importação, preservação, hash, proveniência e análise dos formatos implementados | Parsers adicionais conforme necessidade |
| Registros de autenticação | Importação/preservação; análise quando o formato for suportado | Conectores/parsers específicos |
| Tráfego de rede | Importação/preservação de exportações/artefatos | PCAP e análise especializada |
| Sistemas de monitoramento/SIEM | Importação de exportações ou evidências fornecidas pelo operador | Integrações específicas |
| Endpoints/estações | Importação/preservação de artefatos coletados | Velociraptor/coletores especializados |
| Dispositivos de segurança | Importação/preservação de logs/exportações | Conectores específicos quando necessários |

**Regra de honestidade de escopo:** o projeto não deve afirmar que possui coleta remota ou parsing especializado de uma fonte apenas porque consegue armazenar seu arquivo. O MVP deve declarar exatamente quais formatos são analisados automaticamente e quais são apenas preservados/importados.

Essa distinção mantém aderência à demanda sem transformar o MVP em um SIEM, EDR ou suíte forense universal.

**ESCOPO DE PARSING AUTOMÁTICO DO MVP:** o único formato de log com parsing semântico implementado no MVP é **EVTX (Windows Event Log)**. Outros artefatos e formatos podem ser importados como evidência, preservados, identificados, hasheados e associados à cadeia de custódia, mas não devem ser apresentados pela interface como automaticamente analisados. A análise semântica desses formatos depende de parser futuro ou intervenção manual do analista.

---

# 5. Decisões arquiteturais finais
<!-- @section id="5" slug="decisoes-arquiteturais-finais" -->

## 5.1. Arquitetura geral

**DECISÃO:** utilizar um **monólito modular de implantação, com o engine de análise executando como processo separado dentro do mesmo container no MVP**, evitando microserviços e múltiplos containers de aplicação.

```text
                         HOST / LINUX
┌───────────────────────────────────────────────────────────────────┐
│                        Podman rootless                            │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                         dfir-app                             │  │
│  │                                                             │  │
│  │  Rust + Axum                                          │  │
│  │  Domain / Application / RBAC / Audit / SQLite               │  │
│  │  Worker único                                               │  │
│  │  Typst / Markdown / JSON                                    │  │
│  │                                                             │  │
│  │    Rust application process                                 │  │
│  │       │                                                     │  │
│  │       │ tokio::process::Command() + stdin/stdout + JSONL    │  │
│  │       ▼                                                     │  │
│  │    Rust process                                             │  │
│  │    EVTX / YARA-X / IOC / normalização / correlação         │  │
│  │                                                             │  │
│  │  storage local: DB / evidence / staging / exports          │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

#### 5.1.1. Fronteiras

**Aplicação Rust (`dfir-app`)** é o núcleo da plataforma:

- HTTP/JSON;
- autenticação e sessões;
- autorização/RBAC;
- domínio e casos de uso;
- persistência SQLite;
- auditoria;
- task queue;
- geração dos modelos de relatório;
- exportação/importação;
- políticas de segurança da aplicação;
- criação e supervisão do processo `dfir-engine`.

**Engine Rust (`dfir-engine`)** é o processo especializado de processamento:

- parsing;
- detecção;
- normalização;
- extração de IOCs;
- correlação determinística;
- processamento de grandes volumes;
- validação de limites de entrada;
- produção de resultados estruturados.

Os dois componentes compartilham o ecossistema Rust, mas continuam separados como processos porque o isolamento de parser é uma decisão de segurança operacional, não uma necessidade de linguagem.

O engine não possui responsabilidades de domínio e não deve conhecer:

- autenticação;
- RBAC;
- persistência de negócio;
- NIST Profiles;
- auditoria de usuário;
- sessão;
- políticas de negócio;
- geração final de documentos.

### 5.1.2. Por que Rust no caminho de produção confiável?

**DECISÃO:** `dfir-app` e `dfir-engine` permanecem em Rust. O frontend pode utilizar TypeScript, mas TypeScript é tratado como **código de browser compilado**, não como runtime de servidor. Python e Shell pertencem à classe de **tooling de desenvolvimento/QA** e não fazem parte do runtime de produção do MVP.

A motivação é separar dois objetivos que não precisam ser resolvidos com a mesma linguagem:

1. manter o caminho de produção confiável pequeno, previsível e fortemente tipado;
2. permitir que o frontend e as ferramentas de engenharia usem ecossistemas adequados às suas funções.

A arquitetura de produção continua:

```text
Browser
   ↓ HTTP/JSON + SSE quando necessário
Rust dfir-app
   ↓ stdin/stdout + JSONL
Rust dfir-engine
```

O frontend é compilado previamente:

```text
TypeScript + Deno (build/dev)
        ↓
assets JavaScript estáticos
        ↓
dfir-app
```

As ferramentas de engenharia ficam fora da imagem/runtime final:

```text
CPython + Shell
  ├─ fixtures
  ├─ validação
  ├─ fuzzing/orquestração
  ├─ benchmarks
  └─ verificações de release
```

Essa divisão reduz a superfície do runtime sem exigir que o backend seja forçado a resolver problemas de visualização ou automação de QA com a mesma linguagem.

### 5.1.3. O que a separação por classes de runtime simplifica

A decisão anterior de unificar todo o backend em Rust continua válida para o **backend confiável**. O que muda é a classificação do frontend e do tooling:

- TypeScript não é um backend paralelo;
- Deno não é runtime da aplicação;
- Python não é backend do produto;
- Shell não é mecanismo de execução de ferramentas de produção.

O frontend usa TypeScript somente para interações ricas que não compensem resolver com HTML server-rendered. A Fase 0 não exige uma biblioteca de visualização de terceiros: a interface pode usar um renderer local mínimo, deixando um ponto de adaptação para `vis-timeline` ou D3 em fase posterior, quando houver necessidade demonstrada.

A aplicação continua utilizando tipos Rust, `serde`/`serde_json` e validações explícitas de domínio/aplicação. A fronteira do engine continua sendo validada novamente, porque compartilhar Rust não transforma dados externos em dados confiáveis.

SQLx continua sendo usado sobre SQLite; repositories finos apenas organizam queries SQL explícitas e não constituem ORM.

### 5.1.4. Processo separado continua obrigatório

Mesmo com todo o backend em Rust, o engine continua como subprocesso para:

- impedir que um crash do parser derrube o servidor HTTP;
- impor timeout, cancelamento e limites de saída;
- restringir filesystem e rede com `bubblewrap`;
- manter o processamento de evidência não confiável fora do processo de aplicação;
- preservar uma fronteira de contrato estável para análise especializada.

O processo Rust de aplicação supervisiona `dfir-engine` com `tokio::process::Command`. O projeto não depende de shell intermediário.

### 5.1.4.1. Limites deste isolamento

O isolamento continua sendo **defesa em profundidade**. Rust reduz classes de erros de memória e dá fortes garantias no código seguro, mas não impede automaticamente erros de autorização, lógica, configuração, criptografia, SQL ou análise forense. A segurança continua dependendo do conjunto:

```text
validação
   ↓
processo Rust separado
   ↓
sandbox bubblewrap por job
   ↓
container rootless
   ↓
host
```

Não descrever esta arquitetura como equivalente a VM, segundo host ou garantia absoluta de isolamento.

### 5.1.4.2. Sandbox por job (bubblewrap)

**DECISÃO:** cada invocação do engine roda dentro de um sandbox `bubblewrap` (`bwrap`) com namespaces dedicados, dentro do mesmo container.

A aplicação Rust executa conceitualmente:

```text
Tokio Worker
   ↓
tokio::process::Command
   ↓
bwrap
   ↓
dfir-engine --stdio
```

A política do sandbox continua sendo:

- `--unshare-user`;
- `--unshare-net`;
- `--unshare-uts`;
- `--unshare-ipc`;
- `--unshare-pid`;
- `--disable-userns`;
- `--die-with-parent`;
- bind mount somente de `/input` e `/output`;
- filesystem mínimo necessário para o binário.

**Requisito de falha fechada:** se o sandbox não puder ser criado, o job falha explicitamente. Nunca executar o engine sem sandbox como fallback silencioso.

A aplicação não fornece caminhos físicos do storage ao engine por entrada HTTP. A referência lógica do job é resolvida pela aplicação para os mounts controlados do sandbox.

**Ressalva honesta:** `bubblewrap` cria seus namespaces sem privilégio (via `--unshare-user`), e namespaces sem privilégio ampliam a superfície de código do kernel exposta a um processo não confiável, comparado a não usar namespace algum — isso é uma característica documentada do próprio modelo de ameaças do kernel Linux, não uma falha de configuração do bubblewrap. [11][12][13] O sandbox por job continua sendo defesa em profundidade sobre essa superfície ampliada, não a eliminação dela; Landlock ou outro LSM equivalente pode ser avaliado como camada complementar caso o threat model do projeto exija redução adicional, sem substituir os namespaces já decididos aqui. [14]

### 5.1.4.3. Contrato de execução

O transporte entre `dfir-app` e `dfir-engine` permanece **stdin/stdout + JSONL** porque continua sendo útil como contrato pequeno, testável e independente do restante da aplicação. A mudança para Rust não elimina essa fronteira; apenas elimina a fronteira de linguagem.

Nenhum socket Unix, porta TCP ou API HTTP do engine é introduzido no MVP.

### 5.1.5. Por que não microserviços?

O projeto é uma aplicação local/offline, com SQLite, worker único, baixa necessidade de escala horizontal e forte requisito de simplicidade operacional.

Microserviços acrescentariam:

- rede interna;
- descoberta/endereçamento;
- autenticação entre serviços;
- mais processos;
- mais containers;
- mais pontos de falha;
- maior superfície de ataque;
- maior custo operacional.

Não existe benefício suficiente no MVP para transformar domínio, relatórios, NIST, auditoria, persistência ou o engine em serviços independentes.

### 5.1.6. Princípio de Necessidade Demonstrada

Nenhuma capacidade adicional é implementada preventivamente apenas porque "parece mais robusta".

PCAP, memory forensics, integração ao vivo com Velociraptor, PostgreSQL, KMS/Vault e outras extensões entram em escopo somente quando houver necessidade concreta demonstrada.

### 5.1.7. Princípio de Separação de Acesso

O `dfir-app` não mantém credenciais administrativas para infraestrutura externa no MVP.

Para integrações futuras:

- a aplicação gera artefatos;
- o operador revisa;
- o operador executa;
- o resultado volta como evidência.

Isso evita transformar o próprio aplicativo DFIR em uma conta privilegiada de infraestrutura.

## 5.2. Matriz de decisões arquiteturais

| Preocupação | Decisão | Alternativa rejeitada | Motivo | Reavaliar quando |
|---|---|---|---|---|
| Arquitetura | Monólito modular + aplicação Rust e engine Rust em processos separados | Microserviços / múltiplos containers | Menor complexidade sem perder isolamento processual | Escala/concurrency real exigir |
| Persistência | SQLite + WAL | PostgreSQL no MVP | Operação local/offline simples | Vários escritores ou escala horizontal real |
| Comunicação app-engine | stdin/stdout + JSONL | UDS / HTTP local | Mesma unidade de implantação; menor complexidade | Engine precisar de execução remota/distribuída |
| Backend confiável | Rust | Backend TypeScript separado | Um único runtime confiável no caminho de produção, com isolamento processual preservado | Requisito/benchmark justificar outro runtime no servidor |
| Detecção | YARA-X | YARA clássico como primário | Engine Rust moderno | Requisito de compatibilidade legada |
| Ruleset MVP | Regras próprias/curadas e versionadas | Feed público automático | Reprodutibilidade e controle de licença/ruído | Processo de curadoria estiver maduro |
| PDF | Typst | Pandoc + Typst obrigatório | Menos etapas e dependências | Necessidade editorial exigir outro pipeline |
| Container | Podman rootless | Runtime rootful/Kubernetes | Least privilege e simplicidade local | Ambiente corporativo impor outro runtime |
| Base da imagem | Alpine 3.24.0 + Rust 1.97.1-alpine3.24 no build | Alpine 3.22 | Patch atual compatível e imagem enxuta | incompatibilidade de pacote/security finding comprovada |
| Serviço Linux | Quadlet/systemd | Kubernetes | Adequado a um host local | Necessidade real de múltiplos nós |
| Coleta remota | Artefato + importação manual | API privilegiada ao vivo | Menor risco operacional | Requisito operacional concreto |
| Isolamento do engine | `bubblewrap` por job, dentro do container único | Segundo container + daemon/UDS | Reduz filesystem, namespaces e conectividade disponíveis ao parser sem criar outro serviço; não equivale a VM/host separado | Threat model ou teste concreto exigir fronteira adicional |
| KMS/Vault | Fase posterior | Dependência obrigatória | Não quebrar Air-Gapped/local | Organização fornecer serviço aprovado |
| Assinatura | Ed25519 para artefatos assináveis | “GPG ou qualquer certificado” | Contrato técnico determinístico | Requisito regulatório exigir outro algoritmo/formato |

## 5.3. Contrato de arquitetura

As seções seguintes devem obedecer à separação:

```text
POLÍTICA / DOMÍNIO
        ↓
      dfir-app
        ↓
tokio::process::Command() / JSONL
        ↓
   processo Rust
        ↓
resultado estruturado
        ↓
      dfir-app
        ↓
persistência / relatório / auditoria
```

Nenhuma seção posterior pode voltar a descrever o engine como escritor direto do SQLite, como serviço HTTP ou como container independente no MVP sem atualizar esta seção.

# 6. Stack tecnológica final
<!-- @section id="6" slug="stack-tecnologica-final" -->

| Camada | Escolha | Motivo |
|---|---|---|
| UI base | HTML server-rendered + CSS | Fluxos de servidor simples, offline-ready e sem framework JS de servidor. |
| UI rica | TypeScript compilado para browser | Timeline, filtros e visualizações interativas sem runtime JS de servidor. |
| Frontend toolchain | Deno 2.9 LTS (build/dev) | TypeScript, check, lint e transpile; não entra no runtime de produção. |
| QA/tooling | CPython 3.14.6 + Shell | Fixtures, validação, fuzzing/orquestração, benchmarks e release tooling; fora do runtime do produto. |
| Web | Rust + Axum + Tokio | Backend único em Rust, HTTP assíncrono e middleware via Tower. |
| Templates | Askama + MiniJinja | Askama para templates internos/fixos da aplicação; MiniJinja para CLI e templates deliberadamente dinâmicos/configuráveis. |
| CLI | Clap + MiniJinja | CLI tipada com saídas renderizadas por templates controlados, sem expor capacidades de execução ao template. |
| Validação | `serde`/`serde_json` + validação de domínio | Menos camadas; parsing e validação permanecem dentro do mesmo ecossistema Rust. |
| Domínio | Rust | Um único modelo de domínio e linguagem no backend. |
| Persistência | SQLite + SQLx + repositories finos | SQL explícito, transações e repositories pequenos para centralizar acesso sem adotar ORM. |
| Engine | Rust | Parsing/scan de entrada potencialmente hostil e processamento pesado. |
| EVTX | crate `evtx` | Parser especializado; desempenho será validado pelo benchmark do projeto. |
| Detecção | YARA-X | Engine Rust moderno de regras. |
| Dados tabulares | Polars via API Rust | Processamento tabular dentro do mesmo ecossistema do engine. |
| IOCs | Rust | Extração determinística e eficiente. |
| Relatórios | Typst | PDF offline com pipeline direto e menor quantidade de etapas. |
| Logs estruturados | `tracing` + `tracing-subscriber` | Instrumentação assíncrona nativa do ecossistema Tokio/Axum já adotado, com formatação JSON e correlação de `trace_id`/`request_id`/`job_id`. |
| Aleatoriedade de aplicação | `getrandom 0.4.3` | Bytes aleatórios do sistema para salts, CSRF e tokens; `default-features=false`, somente `std`; `sys_rng` desabilitado para não introduzir `rand_core` 0.10 sem necessidade. |
| HTTP middleware | Tower + `tower-http` | Request ID, tracing, timeouts e headers de segurança reutilizáveis, evitando middleware próprio duplicado. |
| Engine protocol | stdin/stdout + JSONL | Comunicação local direta entre processos, sem socket/porta adicional. |
| Runtime de containers | Podman rootless | Least privilege e operação local. |
| Serviço Linux | Quadlet/systemd | Gerenciamento declarativo de serviços em Linux. |
| Coleta remota | Velociraptor, Fase 4 | Somente quando coleta em escala for necessária. |
| Hardening | Ansible, opcional | Geração de artefatos auditáveis. |
| Scanner adicional | ClamAV, opcional | Cobertura complementar, nunca engine principal. |
| Supply chain | cargo-audit + cargo-deny + cargo-vet + cargo-cyclonedx | Advisories, licenças/fontes, auditoria de dependências e SBOM reproduzível. |
| Testes avançados | proptest + cargo-nextest | Propriedades para invariantes críticas e runner de testes de desenvolvimento. |

### 6.0.1. Classes de runtime e tooling

O projeto distingue explicitamente três classes de componentes:

| Classe | Componentes | Entra na imagem/runtime de produção? | Regra |
|---|---|---:|---|
| **Production Runtime** | `dfir-app`, `dfir-engine`, bubblewrap, Typst e assets estáticos | Sim | Deve ser mínimo, reproduzível e controlado. |
| **Build Toolchain** | Rust/Cargo, Deno, ferramentas de empacotamento frontend | Não | Pode ser mais rica que o runtime; versões e checksums devem ser registráveis. |
| **Engineering/QA Tooling** | CPython, Shell, scripts de teste, fixtures e benchmarks | Não | Não recebe privilégios de produção por padrão. |

**DECISÃO:** minimizar a quantidade de linguagens e runtimes no **caminho de produção**, não proibir linguagens diferentes no desenvolvimento. Essa distinção substitui a interpretação anterior de que todo o projeto deveria usar uma única linguagem.

Deno é a ferramenta de referência do frontend na linha LTS disponível no ciclo de manutenção. A documentação oficial registra a linha 2.9 como LTS de julho de 2026 a 31 de janeiro de 2027. [F-01]

CPython 3.14.6 é a referência de tooling Python deste ciclo. A série 3.14 é estável e o patch 3.14.6 é a manutenção atual verificada em 2026-08-17. [F-02]

Nenhum código de produção deve depender de APIs exclusivas de Deno, Bun ou Node. Python também não deve participar da cadeia de confiança do backend sem uma ADR específica para transformar um analisador em worker de produção.

### 6.1. Critério geral de substituição

Nenhuma tecnologia é sagrada. A substituição é aceitável quando uma alternativa:

1. atende todos os requisitos funcionais;
2. reduz risco ou complexidade;
3. não quebra rastreabilidade;
4. não viola o modelo de segurança;
5. possui licença compatível;
6. possui benchmark/teste que justifique a alteração.

### 6.2. Polars

Polars possui core implementado em Rust e oferece APIs para Python e Rust. A escolha da API Rust nesta arquitetura visa manter o processamento tabular dentro do mesmo runtime; não é permitido afirmar uma vantagem percentual universal entre APIs.

### 6.3. Web e templates Rust

Axum é uma biblioteca de roteamento e tratamento HTTP que se integra ao ecossistema Tokio/Tower e utiliza uma API modular de handlers e middleware. A documentação atual também declara que o crate é implementado em safe Rust. [1]

Askama gera código Rust para os templates em tempo de compilação e fornece escaping automático por padrão para HTML, reduzindo a necessidade de uma camada de template interpretada em runtime. [16]

### 6.3.1. CLI e templates dinâmicos

**DECISÃO:** MiniJinja será utilizado na CLI e somente em superfícies que precisam de templates dinâmicos/configuráveis. Templates internos e fixos da aplicação web permanecem em Askama.

MiniJinja deve receber somente dados preparados pelo chamador. Nenhum template do projeto pode receber acesso a filesystem, shell, rede, ambiente, subprocessos ou APIs privilegiadas. O template é um mecanismo de apresentação, não um mecanismo de execução. A adoção é compatível com a política de dependências permissivas do projeto: MiniJinja 2.24.0 é Apache-2.0.

A coexistência de Askama e MiniJinja não é considerada duplicação arquitetural porque cada engine tem uma fronteira de responsabilidade distinta.

### 6.3.2. Middleware HTTP
### 6.3.3. Geração de aleatoriedade e compatibilidade do ecossistema password-hash

**DECISÃO:** a aplicação utiliza `getrandom 0.4.3` como fonte direta de aleatoriedade para tokens e salts, com apenas as features `std` e sem `sys_rng`. `rand` e `rand_core` não são dependências diretas da aplicação. O `argon2 0.5.3` é consumido com `default-features=false` e apenas `alloc` + `password-hash`; a feature `rand` fica desabilitada.

O salt de senha possui 16 bytes de entropia, é convertido para `SaltString` e só então entregue ao `PasswordHasher`. A escolha evita misturar famílias incompatíveis de `rand`/`rand_core` em código de aplicação. `getrandom 0.4.3` possui features desabilitadas por padrão e a feature `sys_rng`, que reintroduziria uma dependência opcional em `rand_core 0.10`, permanece desabilitada. `getrandom::fill` obtém bytes da fonte aleatória preferida do sistema e retorna erro explicitamente em falha. A documentação atual do Argon2 confirma `SaltString::encode_b64` e o suporte a `PasswordHasher`.

Esta é uma decisão de compatibilidade, não uma preferência estética: quando uma dependência criptográfica transitiva exige uma família RNG específica, a aplicação deve encapsular a geração de entropia em uma fronteira explícita e evitar expor tipos de RNG de diferentes gerações no mesmo módulo.


**DECISÃO:** Tower/Tower-HTTP é a implementação preferencial de middleware HTTP genérico do projeto. `tower-http` será usado, conforme necessidade real, para request IDs, tracing, timeouts e headers de resposta de segurança, substituindo middleware equivalente implementado localmente.

Adoção de features de `tower-http` deve ser mínima e justificada. CORS não é habilitado por padrão. Middleware não substitui autenticação, autorização, CSRF ou política de visibilidade de dados.

### 6.4. Validação e persistência

A desserialização de entradas HTTP e JSONL deve ser feita com `serde`/`serde_json`, seguida de validação das invariantes de aplicação e domínio. Valores externos não devem ser convertidos diretamente em entidades de domínio sem validação.

O mesmo princípio vale para o engine: a aplicação valida o envelope e os resultados, enquanto o engine valida novamente tudo que recebe antes de processar. Compartilhar a linguagem não elimina a fronteira de confiança entre processos.

SQLx será utilizado como camada de acesso ao SQLite. Queries com entrada dinâmica devem usar parâmetros; queries estáticas relevantes podem usar as macros de verificação de SQL quando isso for compatível com o fluxo de build offline/reprodutível do projeto. [15]

A arquitetura não exige um ORM. O objetivo é reduzir abstrações desnecessárias e manter SQL explícito onde isso melhorar rastreabilidade. Para reduzir repetição sem ocultar o comportamento SQL, a aplicação utiliza **repositories finos** na camada de persistência (`src/persistence/repositories.rs`). Esses repositories centralizam consultas parametrizadas, projeções de leitura e fronteiras de transação de entidades operacionais como usuários, sessões e acesso por incidente. Eles **não são um ORM**: não geram consultas, não introduzem um modelo de entidades carregado automaticamente e não escondem semântica de transações ou filtros de segurança. Regras de autorização, políticas de domínio e decisões de negócio continuam na camada de aplicação/security.

**Critério de segurança:** uma query que atravesse uma fronteira de autorização sensível deve permanecer localizada em um repository pequeno e explicitamente chamado pela camada responsável pela decisão. Isso facilita revisão estática, testes e auditoria sem permitir que uma abstração de persistência decida quem pode acessar quais dados.

### 6.5. Logging estruturado

**DECISÃO:** `tracing` + `tracing-subscriber` (formatador JSON) é a biblioteca de logging estruturado do projeto. [35][36]

A seção 34 já define o formato dos eventos e a lista do que nunca deve ser registrado; esta subseção fixa a implementação. `tracing` foi escolhido sobre alternativas como `log`/`env_logger` porque é nativamente assíncrono e instrumenta `Future`s/spans do próprio Tokio sem adaptação adicional — o mesmo ecossistema já usado por Axum — e porque `trace_id` (seção 34) mapeia diretamente para o conceito de *span* da biblioteca, permitindo correlacionar uma requisição HTTP com o `request_id`/`job_id` da mensagem JSONL correspondente (seção 8) sem infraestrutura extra de correlação.

Nível de log em produção é controlado por variável de ambiente (`RUST_LOG`), nunca hardcoded no binário.


### 6.7. Regra de congelamento e validação do grafo de dependências

O `Cargo.lock` é a fonte de verdade do conjunto efetivamente resolvido para um release/teste controlado. Nenhuma alteração de dependência deve ser aprovada apenas com base no número da versão em `Cargo.toml`. Antes de adicionar, remover ou atualizar uma dependência direta, a implementação deve registrar: (1) versão resolvida; (2) features efetivas; (3) duplicações relevantes; (4) licença; (5) advisories; (6) impacto na SBOM; (7) compatibilidade da API utilizada.

O gate mínimo é: `cargo metadata --locked`, `cargo tree --locked -e features`, `cargo tree --locked -d`, `cargo deny check`, `cargo audit`, `cargo vet` quando aplicável e `cargo cyclonedx` para regeneração da SBOM. Mudanças em autenticação, criptografia ou aleatoriedade exigem também revisão específica das famílias `rand`/`rand_core` transitivas e da API efetivamente resolvida.

O princípio é evitar fronteiras de segurança dependentes de wrappers desnecessários. Quando a aplicação precisa apenas de bytes criptograficamente seguros do sistema, deve preferir o menor componente suficiente e manter a configuração de features explícita.

### 6.6. Política de versões — somente versões estáveis mais recentes

**DECISÃO:** o projeto deve utilizar, para cada componente de software sob seu controle, **a versão estável mais recente disponível no momento em que a dependência for selecionada ou atualizada para o release**. Versões antigas, EOL, beta, alpha, release candidate ou snapshots não devem ser escolhidas quando existir uma versão estável mais recente compatível com os requisitos do projeto.

Esta regra vale, no mínimo, para:

- toolchain Rust;
- crates Rust de produção e ferramentas de build/teste;
- Axum, Tokio, Askama, MiniJinja, Tower, `tower-http`, SQLx, Serde, YARA-X, Polars e demais componentes do backend/engine;
- ferramentas de desenvolvimento e supply chain como `cargo-audit`, `cargo-deny`, `cargo-vet`, `cargo-cyclonedx`, `cargo-nextest` e crates de teste como `proptest`;
- CSS e assets JavaScript compilados; Deno é ferramenta de build e CPython/Shell são ferramentas de engenharia, não componentes do runtime distribuído;
- Typst;
- Podman e componentes auxiliares cuja versão seja controlada pelo projeto;
- ferramentas de desenvolvimento e segurança utilizadas pelo pipeline quando houver release estável disponível.

**Importante:** “mais recente” significa **mais recente estável**, e não o maior número publicado em um canal experimental nem a tag flutuante `latest`. O release deve registrar a versão exata efetivamente utilizada.

A política combina atualização recente com reprodutibilidade:

1. selecionar a versão estável mais recente compatível;
2. fixar a versão exata no manifesto/lockfile correspondente;
3. registrar checksum/digest quando aplicável;
4. registrar a versão efetiva no SBOM;
5. testar a combinação completa antes do release;
6. atualizar novamente para a versão estável mais recente quando uma manutenção/release for iniciada;
7. nunca atualizar silenciosamente um ambiente já instalado.

**Não usar `latest` como identificador de release:** tags flutuantes de imagens, artefatos ou dependências não são suficientes para garantir reprodutibilidade. Para containers, utilizar digest; para crates, `Cargo.lock`; para artefatos frontend, versão exata e checksum/proveniência; para a toolchain Rust, uma versão de toolchain explicitamente registrada.

### 6.6.1. Exceção controlada por compatibilidade

Se a versão estável mais recente quebrar um requisito, introduzir uma vulnerabilidade conhecida, remover uma capacidade necessária ou ainda não possuir suporte compatível com o projeto, **não é permitido permanecer indefinidamente em uma versão antiga sem registro**. A equipe deve:

- registrar a incompatibilidade ou risco;
- manter a versão estável mais recente que seja tecnicamente aceitável;
- abrir uma pendência/ADR quando a diferença for arquitetural ou relevante;
- reavaliar na próxima atualização controlada.

Uma versão antiga mantida por exceção é, portanto, **PENDENTE/RESTRIÇÃO documentada**, nunca uma escolha silenciosa.

**Regra para LLM:** uma LLM implementadora não deve escolher uma versão antiga por familiaridade, por exemplo de tutorial ou por conhecimento de treinamento. Antes de adicionar/atualizar uma dependência, deve verificar a versão estável atualmente adotada pelo projeto e atualizar o manifesto/lockfile, testes, SBOM e documentação afetados.

### 6.6.2. Dependências diretas, transitivas e toolchain

Para criptografia e autenticação, a versão estável mais recente compatível não deve ser escolhida apenas pelo número: deve ser avaliada pelo grafo efetivo de features. A adoção de `getrandom 0.4.3` substitui a dependência direta de `rand_core 0.6` na aplicação, mas não constitui uma promessa de eliminar toda ocorrência transitiva de famílias `rand_core` em ferramentas de teste ou outras dependências. O release deve inspecionar `cargo tree --all-features` antes de fixar o conjunto.

A política de "versão mais recente" diferencia o que está sob controle direto do projeto do que é resolvido transitivamente pelo Cargo.

- **Dependências diretas:** devem usar a versão estável mais recente compatível disponível no momento da seleção/atualização do release.
- **Dependências transitivas:** não precisam ser atualizadas manualmente apenas para atingir o maior número de versão publicado. O Cargo deve resolver uma combinação compatível dentro das restrições do grafo, e o resultado efetivo deve ser registrado no `Cargo.lock`.
- **Overrides/patches:** não devem ser usados apenas para forçar versões mais novas. Só são permitidos quando houver justificativa técnica documentada, como correção de vulnerabilidade, bug ou incompatibilidade resolvida.
- **Toolchain Rust:** deve usar a versão estável mais recente que seja compatível com o workspace e suas dependências. A versão efetivamente utilizada no release deve ser registrada.

Isso evita transformar a política de atualização em uma obrigação artificial de controlar manualmente cada dependência transitiva.

### 6.6.3. MSRV e compatibilidade do toolchain

**DECISÃO:** o projeto deve registrar a versão de Rust utilizada no release e, quando aplicável, o MSRV (Minimum Supported Rust Version) declarado pelas dependências ou pelo próprio workspace.

Se a versão estável mais recente de uma dependência exigir um toolchain incompatível com outra dependência necessária, o projeto não deve contornar o problema silenciosamente. Deve registrar a incompatibilidade e aplicar a exceção controlada da seção 6.6.1 até que a combinação possa ser atualizada.

A política de versões não significa "usar uma versão mais nova a qualquer custo"; significa manter o conjunto de dependências no estado estável mais recente **tecnicamente compatível e testado**.

### 6.6.4. Estratégia de build offline do SQLx

**DECISÃO:** a verificação de queries do SQLx deve ser reproduzível em ambiente Air-Gapped e não pode depender de acesso à Internet durante o build.

O workspace deve adotar uma estratégia explícita de preparação e verificação offline das queries, compatível com a versão efetivamente adotada do SQLx. A implementação deve: 

1. gerar e manter os metadados de verificação necessários para as queries durante o processo controlado de desenvolvimento/release;
2. versionar esses metadados junto ao código quando a estratégia escolhida exigir isso;
3. permitir `cargo check`/`cargo build` em ambiente sem acesso externo;
4. executar os testes contra o SQLite local utilizado pelo projeto;
5. registrar no README/Apêndice C o procedimento exato utilizado pela versão de SQLx adotada no release.

A equipe não deve introduzir `sqlx-cli`, `DATABASE_URL` ou outro mecanismo apenas por hábito de tutorial. Eles só entram se forem necessários para a estratégia de build escolhida e forem documentados.

### 6.6.5. Gates de supply chain

**DECISÃO:** o projeto utilizará ferramentas específicas de desenvolvimento/release para transformar políticas de segurança de dependências em gates automatizáveis:

- `cargo audit` para advisories RustSec;
- `cargo deny` para licenças, fontes e políticas de dependências;
- `cargo vet` para política de auditoria das dependências;
- `cargo cyclonedx` para geração do SBOM;
- `cargo nextest` como runner de testes quando disponível no ambiente;
- `proptest` para propriedades/invariantes críticas.

Essas ferramentas não entram no runtime do produto. Na atualização desta especificação, os baselines verificados externamente foram `tower-http 0.7.0`, `tower 0.5.3`, `MiniJinja` na versão efetivamente resolvida pelo `Cargo.lock`, `getrandom 0.4.3`, `proptest 1.11.0`, `thiserror 2.0.20`, `cargo-audit 0.22.2`, `cargo-deny 0.20.2`, `cargo-cyclonedx 0.5.9` e `cargo-nextest 0.9.140`. O release deve registrar as versões exatas efetivamente utilizadas.

O SBOM do repositório é metadado pré-release até ser regenerado pelo pipeline usando o workspace travado; não é permitido editar manualmente o SBOM para mascarar diferenças do grafo de dependências.

### 6.6.6. Configuração tipada e ownership

**DECISÃO:** variáveis de ambiente e demais configurações de processo são lidas uma vez durante o bootstrap e convertidas em uma configuração tipada (`AppConfig`). Código de domínio/aplicação/infraestrutura não deve ler `std::env` diretamente depois do bootstrap. Valores inválidos de configuração devem impedir a inicialização quando a aplicação não puder operar com segurança.

Segredos de bootstrap e chaves não devem participar de `Debug`, tracing ou mensagens de erro de apresentação.

# 7. Aplicação Rust: decisão e limites
<!-- @section id="7" slug="aplicacao-rust-decisao-e-limites" -->

A aplicação web, domínio, fila, persistência e orquestração serão implementados em Rust.

Responsabilidades principais:

- HTTP e HTML server-rendered;
- autenticação e sessão;
- autorização;
- domínio e casos de uso;
- SQLite;
- auditoria;
- task queue;
- criação/supervisão do engine;
- modelos de relatório;
- exportação/importação.

O frontend continua sendo responsabilidade de HTML server-rendered + TypeScript + CSS próprio da aplicação.

## 7.1. Runtime assíncrono

Tokio é o runtime assíncrono do backend Rust. Axum opera sobre Tokio, e tarefas que bloqueiem significativamente o runtime devem usar `spawn_blocking` ou workers dedicados. [1]

O sistema continua com um único job pesado por vez no MVP; `async` não equivale a paralelismo irrestrito.

## 7.2. Segurança de processo

Subprocessos devem ser criados com `tokio::process::Command`, com caminho absoluto/confiável, argumentos estruturados e sem shell intermediário. [10]

Timeout, cancelamento, consumo de stdout/stderr e término do processo pertencem ao Worker da aplicação.

---

# 8. Motor Rust
<!-- @section id="8" slug="motor-rust" -->

## 8.1. Responsabilidades

O engine é um executor especializado.

Responsabilidades:

- parsing EVTX;
- parsers posteriores;
- YARA-X;
- extração de IOCs;
- normalização;
- correlação determinística;
- validação de limites;
- resultados estruturados.

Não é responsabilidade do engine:

- autenticação;
- RBAC;
- persistência de negócio;
- NIST;
- auditoria de usuários;
- gestão de sessão;
- geração final de PDF.

## 8.2. Comunicação de produção

**DECISÃO:** o transporte de produção será **stdin/stdout do subprocesso Rust**, com protocolo JSONL.

```text
Rust Worker
   │
   │ stdin: REQUEST JSON
   ▼
dfir-engine
   │
   │ stdout: PROGRESS / RESULT JSON
   ▼
Rust Worker
```

Não existe socket Unix, porta TCP ou API HTTP entre o app e o engine no MVP.

### Estrutura

O contrato de mensagens permanece o da versão anterior, incluindo `protocol_version`, `schema_version`, `message_type`, `request_id`, `job_id`, limites e referências lógicas de entrada/saída.

### Regras

- uma execução do subprocesso corresponde a um job;
- o Worker mantém um job pesado por vez no MVP;
- nenhuma mensagem carrega a evidência inteira;
- `input_reference` é uma referência lógica interna;
- cada mensagem deve possuir `protocol_version`, `schema_version`, `message_type`, `request_id` e `job_id`;
- tamanho máximo das linhas JSONL deve ser configurável;
- stdout contém exclusivamente mensagens JSONL do protocolo; diagnóstico textual pertence ao stderr;
- timeout e limite de saída possuem efeito real no Worker;
- limites de eventos/itens de resultado são verificados pelo engine e pela aplicação quando aplicável;
- resultados semanticamente inválidos são rejeitados pela aplicação;
- o processo é encerrado pelo Worker quando exceder timeout ou limite definido;
- erro de protocolo não reinicia o job silenciosamente.

## 8.3. Modo de desenvolvimento/teste

O mesmo protocolo JSONL é usado em produção e em testes. A diferença é apenas a forma de iniciar o processo:

```text
produção:
Rust Worker → tokio::process::Command → dfir-engine

teste/desenvolvimento:
CLI → dfir-engine --stdio
```

O `--stdio` continua sendo uma interface suportada do engine.

## 8.4. Limite de confiança do staging

Qualquer artefato escrito pelo engine em `staging/` deve ser tratado pela aplicação como **dado não confiável** até validação.

A aplicação deve validar:

- schema;
- caminho e raiz permitida;
- tamanho;
- tipo esperado;
- checksum quando aplicável;
- associação ao `job_id`;
- associação ao `evidence_id`.

O engine não pode escrever em `db/`, `exports/` ou em caminhos fora do staging autorizado.

## 8.5. Segurança do protocolo

O JSONL é uma fronteira de confiança entre dois processos do mesmo produto.

- requests são validados antes de processar;
- resultados são validados antes de persistir;
- mensagens acima do limite são rejeitadas;
- linhas malformadas provocam falha controlada do job, não do servidor HTTP;
- versões de protocolo divergentes geram erro explícito;
- `input_reference` nunca é um caminho físico fornecido pelo navegador;
- o app resolve a referência lógica para o mount `/input` do sandbox;
- o engine não precisa conhecer a estrutura física de `storage/`;
- nenhum subprocesso é executado via shell;
- stdout e stderr são lidos de forma assíncrona para evitar deadlock por pipe cheio;
- o Worker mata o processo quando o limite de execução é atingido;
- o engine possui um limite máximo de saída por job.

## 8.6. Runtime assíncrono

Tokio continua sendo usado para leitura/escrita assíncrona do protocolo e emissão de progresso. Operações pesadas devem utilizar `spawn_blocking` ou threads dedicadas.

## 8.7. Tolerância a falhas

No código de produção:

- evitar `unwrap()` e `expect()` em caminhos de entrada;
- `panic!` não deve ser usado como fluxo de erro esperado;
- erros devem ser modelados com `Result<T, E>` e tipos de erro explícitos;
- falha do engine deve produzir `FAILED`/`TIMEOUT` no job e não derrubar o servidor web;
- falha de um registro individual de EVTX não deve interromper o arquivo inteiro.

Rust fornece garantias fortes no código seguro, mas a especificação não considera isso substituto para validação, testes ou sandbox.

---

# 9. YARA-X
<!-- @section id="9" slug="yara-x" -->

YARA-X será o engine primário de detecção baseada em regras. O projeto YARA-X é uma reimplementação em Rust do ecossistema YARA e é projetado para segurança, desempenho e usabilidade. [2]

Regras:

- match não é prova definitiva de malware;
- registrar rule ID;
- registrar versão do engine;
- registrar `ruleset_id` e `ruleset_version`;
- registrar hash do ruleset;
- validar regras antes de ativá-las;
- possuir testes positivos e negativos;
- versionar regras próprias.

## 9.1. Ruleset do MVP

**DECISÃO:** o MVP utilizará **ruleset próprio e curado**, versionado no repositório/distribuição.

Motivos:

- demonstração determinística;
- licença e proveniência controladas;
- comportamento reproduzível;
- revisão humana das regras;
- ausência de dependência de atualização automática.

Fontes públicas, incluindo YARA-Forge, podem ser utilizadas como **entrada opcional de pesquisa/curadoria**, mas não como feed automático obrigatório do MVP.

YARA-Forge possui seu próprio código/projeto sob GPL-3.0, mas isso **não significa que todas as regras agregadas devam ser tratadas automaticamente como GPL-3.0**; as regras de origem podem manter termos próprios. Portanto, qualquer regra importada deve conservar sua proveniência e os termos aplicáveis àquela regra/fonte antes de entrar no ruleset do projeto. [17]

## 9.2. Processo de curadoria

A curadoria deve seguir um fluxo determinístico:

```text
Fonte pública / regra proposta
        ↓
Triagem de relevância
        ↓
Revisão manual de conteúdo e licença
        ↓
Teste positivo
        ↓
Teste negativo / falso positivo conhecido
        ↓
Revisão
        ↓
Aprovação
        ↓
Ruleset versionado + hash
        ↓
Release
```

**DECISÃO:** o MVP não realizará atualização automática de regras. A criação de uma nova versão de ruleset ocorre por evento relevante ou ciclo de manutenção definido pela equipe, nunca por obrigação de calendário arbitrário.

Cada alteração deve registrar:

- regra adicionada, removida ou modificada;
- origem;
- licença/termos aplicáveis;
- justificativa;
- analista/revisor;
- resultado dos testes;
- data de aprovação;
- versão e SHA-256 do ruleset resultante.

Uma frequência trimestral pode ser utilizada como **cadência operacional opcional**, mas não é requisito do MVP.

## 9.3. Proveniência

Para cada ruleset:

```text
ruleset_id
ruleset_version
source
source_version/date
license
sha256
engine_compatibility
```

## 9.4. Cache

A chave mínima é:

```text
sha256_arquivo
+
ruleset_id
+
ruleset_version
+
engine_version
```

Nunca cachear exclusivamente pelo SHA-256 do arquivo.

# 10. Parsing EVTX
<!-- @section id="10" slug="parsing-evtx" -->

## 10.1. Integridade

Um parser nunca deve fazer:

```text
except → continue
```

sem registrar a falha.

Falhas devem possuir:

- `evidence_id`;
- referência do record;
- tipo do erro;
- mensagem sanitizada;
- timestamp, se disponível;
- parser version.

Em código, isso se traduz no padrão de erro definido na seção 8.5: a falha de um registro vira uma variante de `EngineError`, é registrada estruturadamente, e o loop de parsing segue para o próximo registro — nunca `panic!`, nunca abortar o arquivo inteiro por um registro ruim.

### 10.1.1. Formatos de log suportados

O MVP oferece parsing automático para **EVTX (Windows Event Log)**.

Os seguintes formatos/artefatos **não possuem parsing automático no MVP**, entre outros que não estejam explicitamente cobertos pela seção 10:

- PCAP/PCAPNG (tráfego de rede);
- Syslog (incluindo RFC 3164/5424);
- logs de servidores web (Apache, Nginx, IIS);
- logs de firewall e dispositivos de rede;
- arquivos de Registro do Windows (`.reg` e hives);
- Prefetch (`.pf`);
- EVT (formato legado do Windows);
- logs de aplicações em formatos proprietários.

Esses formatos podem ser **importados como evidência**, preservando o arquivo original, hash, proveniência e cadeia de custódia conforme as regras gerais de evidência. A importação não implica interpretação semântica nem correlação automática. Quando o formato não for suportado por um parser do MVP, a análise dependerá de parser futuro ou de intervenção manual do analista.

Novos parsers devem ser avaliados segundo o **Princípio de Necessidade Demonstrada** (seção 5.1.6), incluindo necessidade real, segurança do parser, desempenho, manutenção e licenciamento. O catálogo de candidatos para fases posteriores está no **Apêndice E** e não constitui compromisso de implementação.

**CRITÉRIO DE ACEITAÇÃO:** a aplicação deve distinguir visual e semanticamente entre evidência **preservada/importada** e evidência **analisada por parser automático**. Nenhum formato sem parser implementado pode ser apresentado como automaticamente analisado.

## 10.2. Resultado mínimo

```json
{
  "event_id": "4688",
  "timestamp": "...",
  "provider": "Microsoft-Windows-Security-Auditing",
  "computer": "HOST-01",
  "user_reference": "user_...",
  "process": "powershell.exe",
  "command_line": "...",
  "ip_address": "...",
  "evidence_id": "ev_...",
  "raw_reference": "record:12345",
  "normalization_status": "NORMALIZED"
}
```

### Parsing em streaming

Arquivos grandes devem ser tratados incrementalmente/batch.

Não assumir que o arquivo inteiro cabe em memória.

---

# 11. PCAP e memória
<!-- @section id="11" slug="pcap-e-memoria" -->

## 11.1. PCAP

**FASE POSTERIOR** — aplica-se o Princípio de Necessidade Demonstrada.

Opções técnicas a validar no momento da implementação:

- `pcap`;
- `etherparse`;
- `pnet`.

`Scapy` é Python e não deve ser descrito como crate Rust.

O parser futuro deve suportar limites explícitos de:

- bytes;
- pacotes;
- duração;
- memória;
- payload materializado.

Não descriptografar TLS sem material criptográfico autorizado.

## 11.2. Memory Dump

**FASE POSTERIOR** — usar ferramenta especializada por subprocesso/container quando houver caso de uso concreto.

Não reimplementar Volatility em Rust apenas por preferência tecnológica.

## 11.3. Velociraptor

Integração futura por:

1. geração de artefato/consulta;
2. download pelo operador;
3. execução autorizada pelo operador;
4. exportação do resultado;
5. upload pelo mesmo pipeline de evidência;
6. hash e chain of custody;
7. análise.

O `dfir-app` não mantém credenciais para a infraestrutura remota e não acessa a API do Velociraptor no MVP.

Uma integração ao vivo somente poderá ser introduzida após requisito operacional concreto e revisão de segurança específica.

# 12. ClamAV
<!-- @section id="12" slug="clamav" -->

**OPCIONAL e desligado por padrão.**

Função:

- segunda opinião de detecção;
- cobertura complementar de assinatura;
- nunca substituir YARA-X;
- nunca interpretar `CLEAN` como prova de ausência de ameaça.

Requisitos:

- limite de tamanho;
- timeout;
- isolamento;
- resultado `CLEAN`, `DETECTED`, `ERROR`, `UNAVAILABLE`, `TIMEOUT`.

### Rulesets de terceiros

Rulesets externos não devem ser incorporados silenciosamente à distribuição principal.

Quando usados:

1. registrar fonte;
2. versão/data;
3. checksum;
4. licença/termos aplicáveis;
5. habilitação explícita pelo administrador;
6. manter separação entre rules oficiais e terceiros.

Não usar alegações de marketing de terceiros como requisito de performance ou eficácia.

### Gerenciamento de assinaturas de terceiros — `clamav-unofficial-sigs`

`clamav-unofficial-sigs` é uma ferramenta de atualização de bases não oficiais e suporta várias fontes, incluindo Sanesecurity e outras. [18]

**DECISÃO:** nenhuma fonte de assinatura de terceiros será tratada como padrão obrigatório do MVP. Sanesecurity pode ser usada como fonte opcional, mas seus próprios termos não autorizam que o projeto a classifique genericamente como "BSD" ou redistribua suas assinaturas sem observar as condições da fonte. [19]

Quando uma fonte externa for habilitada:

1. registrar fonte, versão/data, checksum e termos aplicáveis;
2. habilitar explicitamente pelo administrador;
3. manter a base fora do ruleset oficial YARA-X;
4. validar falsos positivos e comportamento no dataset do projeto;
5. documentar a fonte efetivamente escolhida no manifesto do release.

### Atualização offline de assinaturas

Em modo Air-Gapped, a máquina isolada **não baixa assinaturas**. O fluxo é:

```text
máquina conectada
      ↓
baixar/testar assinaturas
      ↓
registrar fonte + versão + checksum + termos
      ↓
pacote de atualização
      ↓
mídia controlada
      ↓
máquina isolada
      ↓
verificação + instalação em `rules/clamav/`
      ↓
reload/restart controlado do ClamAV
      ↓
auditoria
```

O script de atualização é ferramenta operacional, não autoridade de licenciamento nem de proveniência. Atualização automática deve permanecer desligada no modo Air-Gapped.

---

# 13. NIST CSF 2.0
<!-- @section id="13" slug="nist-csf-2-0" -->

A implementação deve representar:

```text
CSF Core
├── Govern (GV)
├── Identify (ID)
├── Protect (PR)
├── Detect (DE)
├── Respond (RS)
└── Recover (RC)

Organizational Profiles
├── Current Profile
└── Target Profile

Tiers
├── Tier 1 — Partial
├── Tier 2 — Risk Informed
├── Tier 3 — Repeatable
└── Tier 4 — Adaptive
```

A organização do Core em Functions, Categories e Subcategories segue a estrutura oficial do NIST CSF 2.0. [6] NIST define Current e Target Profiles como representações da postura atual/desejada em termos dos outcomes do Core. Tiers contextualizam o rigor das práticas de governança e gestão de risco; não são simplesmente uma nota de maturidade 1–4. [3][4]

## 13.1. Gap Analysis

```text
Current Profile
      ↓
Evidence / Assessment
      ↓
Gap
      ↓
Target Profile
      ↓
Prioritization
      ↓
Remediation
```

Cada avaliação deve registrar:

- outcome/subcategory;
- estado atual;
- estado alvo;
- evidência;
- justificativa;
- risco;
- prioridade;
- recomendação;
- confiança.

## 13.2. Tiers e status de implementação

A aplicação deve registrar o **Tier como contexto do Organizational Profile**, com justificativa e evidências.

Não representar Tier como score oficial por Function, Category ou Subcategory.

Para permitir uma visão operacional de "maturidade/progresso" sem distorcer o NIST, cada outcome/Subcategory pode possuir um status separado:

```text
IMPLEMENTED
PARTIALLY_IMPLEMENTED
NOT_IMPLEMENTED
NOT_ASSESSED
NOT_APPLICABLE
```

Esse status é uma convenção operacional do produto, não um nível oficial do NIST.

Assim:

```text
NIST Tier
    ↓
contexto organizacional de governança/risco

Implementation Status
    ↓
estado operacional de cada outcome/subcategory
```

A apresentação pode derivar percentuais/gráficos desses status, desde que nunca sejam rotulados como "Tier NIST por Subcategory". O documento pode usar o termo **maturidade operacional** apenas como convenção de apresentação; o valor não é um score oficial do NIST.

---

# 14. MITRE ATT&CK
<!-- @section id="14" slug="mitre-att-ck" -->

Um mapping só existe quando existir evidência suficiente.

Modelo:

```text
Evidence
   ↓
Finding
   ↓
ATT&CK Mapping
├── tactic
├── technique
├── sub-technique
├── basis
├── confidence
└── analyst_note
```

Confiança:

```text
LOW
MEDIUM
HIGH
CONFIRMED
```

O sistema não pode inventar mappings para tornar o relatório mais impressionante.

---

# 15. Análise de caminho crítico
<!-- @section id="15" slug="analise-de-caminho-critico" -->

A plataforma deve representar o encadeamento do incidente como grafo temporal/causal quando os dados permitirem.

Exemplo:

```text
Initial Access
    ↓
Credential Compromise
    ↓
Execution
    ↓
Privilege Escalation
    ↓
Lateral Movement
    ↓
Collection
    ↓
Exfiltration Attempt
```

Cada transição deve possuir evidência e confiança.

A ausência de evidência deve ser exibida como lacuna, não preenchida por inferência silenciosa.

---

# 16. Matriz de impacto
<!-- @section id="16" slug="matriz-de-impacto" -->

A matriz deve separar pelo menos:

- impacto em disponibilidade;
- impacto em confidencialidade;
- impacto em integridade;
- número/tipo de ativos afetados;
- dados afetados;
- interrupção operacional;
- impacto financeiro quando conhecido;
- confiança da estimativa.

Não fabricar valores financeiros quando não houver dados.

---

# 17. IOC extraction
<!-- @section id="17" slug="ioc-extraction" -->

Tipos mínimos:

- IP;
- domínio;
- URL;
- e-mail;
- SHA-256;
- SHA-1, quando presente na fonte;
- MD5 apenas por interoperabilidade;
- caminho de arquivo;
- hostname.

Cada IOC deve guardar:

- origem;
- evidência;
- evento;
- confiança;
- primeira ocorrência;
- última ocorrência.

---

# 18. Sigma
<!-- @section id="18" slug="sigma" -->

A geração de regras Sigma é **recurso de apoio**.

A LLM ou motor não deve criar regra com lógica inventada.

Cada regra deve apontar para:

- finding de origem;
- evento(s) que sustentam a regra;
- logsource;
- campo(s);
- rationale;
- nível de confiança.

Regras devem ser validadas sintaticamente antes da exportação.

---

# 19. Preservação e cadeia de custódia
<!-- @section id="19" slug="preservacao-e-cadeia-de-custodia" -->

## 19.1. Regra fundamental

```text
Evidência original
       ↓
Cópia de trabalho
       ↓
Normalização
       ↓
Sanitização opcional
       ↓
Análise
```

A original nunca deve ser modificada.

## 19.2. Hash

SHA-256 é o digest principal.

MD5 só pode aparecer por interoperabilidade e nunca como mecanismo principal de integridade.

## 19.3. Chain of Custody

Cada entrada:

```text
entry_id
previous_entry_hash
action
actor
timestamp
reason
metadata
entry_hash
signature_reference
```

O hash da entrada deve depender de uma representação canônica dos campos. Para o MVP, a canonização deve ser especificada e testável: UTF-8, nomes de campos determinísticos, ordenação determinística das chaves, representação estável de valores ausentes e timestamp em formato normalizado. O material que será assinado deve possuir um identificador de versão do formato para impedir ambiguidades entre releases.

A chain of custody é **tamper-evident**, não inviolável: um operador que obtenha acesso suficiente ao banco pode tentar alterar tanto dados quanto a cadeia. A assinatura digital, quando habilitada, aumenta a capacidade de detectar adulteração fora do banco, mas não substitui controles de acesso, backup e separação de chaves.

## 19.4. Assinatura

**DECISÃO:** assinatura digital da cadeia é recomendada para o modo forense; deve ser implementada com um algoritmo/formato único definido pelo projeto, em vez de permitir “GPG ou qualquer certificado” no contrato de produção.

A chave privada deve ficar separada do storage de evidências sempre que possível.

A verificação deve ser obrigatoriamente implementada se a assinatura for habilitada.

---

# 20. Upload, arquivos e `.dfirpkg`
<!-- @section id="20" slug="upload-arquivos-e-dfirpkg" -->

## 20.1. Upload

Requisitos:

- UUID interno;
- caminho físico nunca baseado no nome original;
- validação de tamanho;
- validação de extensão/tipo como defesa em profundidade;
- armazenamento fora da raiz pública;
- permissões mínimas;
- hash imediato;
- registro de ator e horário;
- conteúdo nunca executado automaticamente.

## 20.2. Arquivos compactados

Além de path traversal, proteger contra:

- Zip Slip;
- symlinks;
- archive bombs;
- compressão extrema;
- número excessivo de arquivos;
- profundidade de diretórios;
- tamanho total descompactado.

Devem existir limites configuráveis para:

```text
max_input_bytes
max_extracted_bytes
max_files
max_depth
max_compression_ratio
```

## 20.3. `.dfirpkg`

O pacote de exportação/importação deve possuir:

```text
manifest.json
incident.json
evidence-manifest.json
chain-of-custody.json
evidence/ (opcional)
reports/ (opcional)
schema-version
package-version
checksums
signature (opcional)
```

A importação deve ocorrer primeiro em uma área temporária/quarentena controlada. A aplicação deve validar manifestação, checksums, assinatura quando habilitada, limites de tamanho/quantidade, referências internas e segurança de extração antes de promover qualquer artefato para armazenamento permanente ou persistir objetos no banco.

Nunca extrair diretamente um `.dfirpkg` não confiável dentro de `evidence/`, `db/`, `exports/` ou outras raízes permanentes.

---

# 21. Privacidade, proteção de dados e retenção
<!-- @section id="21" slug="privacidade-e-lgpd" -->

Três estados:

```text
FORENSIC
SANITIZED
DEMO
```

## 21.1. FORENSIC

Preserva máxima fidelidade.

## 21.2. SANITIZED

Cópia derivada para compartilhamento/uso controlado.

## 21.3. DEMO

Dataset sintético ou autorizado, sem segredo real e sem PII real quando não necessária.

Pseudonimização deve preservar correlação quando necessário, sem transformar hash truncado de username em suposta anonimização definitiva.

## 21.4. Retenção e descarte

A demanda exige confidencialidade e privacidade, mas não define prazos de retenção nem uma política jurídica específica. Portanto, esta seção define **mecanismos técnicos de governança de dados**, não uma interpretação jurídica nem um prazo universal de retenção.

**DECISÃO:**

- **FORENSIC**: mantido enquanto o incidente estiver aberto ou sob obrigação legal/contratual de retenção (ex.: exigência regulatória do cliente). Não há prazo fixo no MVP porque isso depende do contexto de cada organização — mas o sistema deve registrar, por incidente, uma data de revisão de retenção, não deixar indefinido silenciosamente;
- **SANITIZED**: revisão a cada encerramento de caso; sem justificativa registrada para manter, é candidato a descarte;
- **DEMO**: não tem dado real, não está sujeito a esta política — mas deve continuar rotulado como tal para nunca ser confundido com FORENSIC/SANITIZED em auditoria;
- **descarte** é uma ação auditada (seção 34), executável apenas por papel ADMIN, e nunca some sem deixar rastro: o evento de exclusão fica no log de auditoria mesmo depois de o dado em si ter sido apagado;
- **legal hold**: se um incidente estiver sob investigação/processo formal, a exclusão automática ou por prazo deve poder ser suspensa explicitamente. **PENDENTE:** depende de validação com o cliente antes de virar requisito rígido — mas o modelo de dados não deve impedir essa flag de existir.

Esta seção define diretriz, não calendário fixo. **PENDENTE:** o prazo exato de retenção é decisão de negócio de cada organização usuária, não do software.

---

# 22. Modelo de dados
<!-- @section id="22" slug="modelo-de-dados" -->

## 22.1. Entidades

```text
Incident
├── Evidence
│   ├── EvidenceHash
│   ├── ChainOfCustodyEntry
│   └── Artifact
├── Event
│   ├── Source
│   └── ATTACKMapping
├── Finding
│   ├── Risk
│   ├── EvidenceReference
│   └── Remediation
├── Assessment
│   ├── CurrentProfile
│   ├── TargetProfile
│   ├── Gap
│   └── TierAssessment
├── IOC
├── TimelineAnnotation
├── IncidentAccess
└── AuditEvent
```

## 22.2. Incidentes múltiplos

A aplicação deve suportar múltiplos incidentes sem misturar evidências, findings, NIST assessments ou relatórios.

Cada objeto de investigação deve possuir `incident_id` quando aplicável.

## 22.3. Evidence

Campos mínimos:

- `id`;
- `incident_id`;
- `original_filename`;
- `storage_key`;
- `mime_type`;
- `size_bytes`;
- `sha256`;
- `source`;
- `collection_method`;
- `collection_tool`;
- `collection_tool_version`;
- `authorization_reference`;
- `collected_at`;
- `ingested_at`;
- `ingested_by`;
- `preservation_status`.

**Não usar `deleted_at` como substituto de governança de evidência.**

Estados sugeridos:

```text
ACTIVE
ARCHIVED
QUARANTINED
DESTROYED
```

`DESTROYED` é uma operação administrativa excepcional e auditada.

---

## 22.4. AuditEvent

A auditoria do aplicativo deve ser **tamper-evident**, mesmo permanecendo no SQLite.

Campos mínimos:

```text
id
timestamp
actor_id
action
object_type
object_id
incident_id (quando aplicável)
metadata
previous_hash
event_hash
```

`event_hash` depende da representação canônica do evento e de `previous_hash`.

A verificação deve permitir detectar:

- alteração;
- remoção;
- reordenação;
- quebra da cadeia.

A auditoria de negócio não é substituta da chain of custody das evidências; são trilhas diferentes.

A cadeia de `AuditEvent` também é tamper-evident, não tamper-proof. Para elevar a garantia além do SQLite local, a arquitetura pode exportar checkpoints assinados da cadeia; isso é opcional no MVP e só deve ser implementado se houver necessidade demonstrada.
# 23. RBAC e segurança do aplicativo
<!-- @section id="23" slug="rbac-e-seguranca-do-aplicativo" -->

Papéis mínimos:

```text
ADMIN
ANALYST
VIEWER
AUDITOR
```

Além do papel global, usar autorização por incidente:

```text
IncidentAccess
├── user_id
├── incident_id
├── permission
├── granted_at
└── granted_by
```

`permission` deve ser uma enumeração controlada pelo domínio, não uma string arbitrária fornecida pela interface. O MVP deve possuir pelo menos:

```text
INCIDENT_VIEW
EVIDENCE_READ
EVIDENCE_IMPORT
ANALYSIS_EXECUTE
REPORT_EXPORT
ARTIFACT_EXPORT
ASSESSMENT_EDIT
ACCESS_MANAGE
AUDIT_READ
AUDIT_VERIFY
EVIDENCE_DESTROY
RULESET_MANAGE
SYSTEM_UPDATE
```

A autorização efetiva resulta de **papel global + acesso ao incidente + permissão da operação**. Nenhuma rota deve depender apenas do papel global quando manipular dados de um incidente específico.

**SEGURANÇA:** uma rede privada ou localhost reduz exposição externa, mas não elimina risco interno. RBAC, autorização por incidente e auditoria continuam sendo controles de integridade do processo investigativo. Esses controles não substituem defesa de rede; eles protegem a confiabilidade e rastreabilidade do resultado da investigação.

**DECISÃO DE CONFIDENCIALIDADE:** autorização de leitura não implica autorização de leitura do conteúdo bruto. O produto separa **permissão de operação** de **nível máximo de visibilidade**. Cada concessão `IncidentAccess` possui `max_visibility`, com os níveis `SUMMARY`, `SANITIZED`, `DETAILED` e `RAW`.

Teto por papel no MVP:

```text
VIEWER  → SUMMARY
AUDITOR → SANITIZED
ANALYST → DETAILED
ADMIN   → RAW
```

O teto do papel não pode ser ultrapassado por uma concessão de incidente. Um endpoint que exponha conteúdo deve verificar a permissão da operação **e** o nível de visibilidade solicitado.

**VIEWER** é consulta operacional mínima e não recebe `AUDIT_READ`, `AUDIT_VERIFY` nem `EVIDENCE_READ`; sua consulta é limitada a superfícies `SUMMARY`.

**AUDITOR** é um papel de fiscalização independente: recebe `AUDIT_READ` e `AUDIT_VERIFY`, não recebe `EVIDENCE_READ`, `EVIDENCE_IMPORT`, `ANALYSIS_EXECUTE`, `ASSESSMENT_EDIT`, `EVIDENCE_DESTROY` ou `SYSTEM_UPDATE`. O acesso do auditor ao conteúdo do incidente é limitado ao nível `SANITIZED`, quando uma operação de consulta justificar essa exposição.

**ANALYST** recebe os dados necessários à investigação dentro do teto `DETAILED`; conteúdo `RAW` permanece restrito ao `ADMIN` no modelo base.

**Nota de implementação:** `SANITIZED` significa uma projeção explicitamente preparada para compartilhamento controlado. Não é permitido tratar “somente leitura” como sinônimo de “conteúdo seguro”. Dados sensíveis como credenciais, tokens, segredos, campos privados desnecessários e detalhes operacionais de exploração devem ser excluídos ou redigidos antes de uma projeção `SANITIZED`.

### 23.1. Visibilidade e minimização de dados

O fluxo de consulta sensível deve seguir:

```text
rota/comando
   ↓
autorização da operação
   ↓
IncidentAccess
   ↓
teto do papel
   ↓
max_visibility da concessão
   ↓
projeção de dados permitida
```

Uma função de leitura não deve receber um objeto bruto e decidir posteriormente o que esconder. A projeção autorizada deve ser escolhida na fronteira de aplicação antes que dados além do nível permitido atravessem para a camada de apresentação.

A ausência de uma rota de evidência na Fase 0 não autoriza removê-la da arquitetura: qualquer Fase posterior que exponha evidência deve adotar esse contrato de visibilidade.

## 23.2. Bootstrap de administrador

Não utilizar senha administrativa padrão.

Primeira execução:

```text
Aplicação
 ↓
Bootstrap controlado
 ↓
Criar primeiro administrador
 ↓
Senha definida pelo operador
```

---

# 24. Autenticação e sessão
<!-- @section id="24" slug="autenticacao-e-sessao" -->

Obrigatório:

- **Argon2id como mecanismo padrão de derivação de senha**, com parâmetros versionados e testados; bcrypt somente por necessidade de compatibilidade documentada;
- cookies `HttpOnly`;
- `Secure` quando HTTPS;
- `SameSite` adequado;
- expiração;
- logout;
- proteção contra session fixation;
- rate limiting;
- proteção de endpoints sensíveis;
- proteção CSRF em toda rota de mutação (POST/PUT/PATCH/DELETE) acessível a partir de sessão de navegador; dispensável apenas se `SameSite=Strict` for usado de forma comprovadamente suficiente para o fluxo em questão, com justificativa registrada.

## 24.1. Rate limiting

Evitar bloqueio permanente de conta que permita negação de serviço contra usuários legítimos.

**DECISÃO:** aplicar backoff progressivo associado à conta e ao contexto de origem:

- após 5 falhas consecutivas: atraso de 1 minuto;
- novas falhas: atraso progressivo até um teto configurado;
- sucesso zera a sequência;
- respostas externas permanecem genéricas para não revelar existência de conta;
- tentativas bloqueadas são auditadas;
- limites adicionais por origem podem existir como defesa contra ataques distribuídos.

A política exata deve ser configurável e testada contra cenários de brute force e account lockout DoS. Em instalação estritamente local/privada, o perfil de rate limiting pode ser menos agressivo; entretanto, a proteção de autenticação não deve ser removida apenas porque o serviço está em localhost. Se a aplicação for exposta além do loopback, o perfil de limitação deve ser reforçado e validado para o ambiente.

# 25. Self-hardening
<!-- @section id="25" slug="self-hardening" -->

Toda entrada proveniente de evidência, log, arquivo ou usuário é não confiável.

## 25.1. XSS

- escaping contextual;
- nenhum HTML bruto com dados não confiáveis;
- CSP;
- validação de saída.

CSP inicial:

```text
default-src 'self';
script-src 'self';
style-src 'self';
img-src 'self' data:;
connect-src 'self';
object-src 'none';
base-uri 'self';
form-action 'self';
frame-ancestors 'none'
```

Sem `'unsafe-inline'` por padrão.

## 25.2. Command injection

Subprocessos devem receber argumentos estruturados, sem shell.

```rust
Command::new("/usr/bin/example-tool")
    .arg("--input")
    .arg(safe_path)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
```

O exemplo é deliberadamente genérico. Em produção, o binário deve ser um caminho fixo/permitido e os argumentos devem ser estruturados. O `dfir-engine` é iniciado pelo Worker com `tokio::process::Command`, usando o protocolo JSONL definido na seção 8.

## 25.3. Path traversal

- raízes permitidas;
- resolução do caminho;
- rejeição de escape da raiz;
- política de symlink;
- IDs internos;
- validação pós-resolução.

## 25.4. Segredos

Segredos de runtime que o app realmente necessita devem ser fornecidos por mecanismo de segredo do ambiente, preferencialmente `podman secret` como arquivo montado, nunca codificados no repositório ou na imagem. [5]

Não colocar:

- chaves privadas;
- tokens;
- segredos de sessão;
- credenciais externas

em `Environment=` simples.

Chaves de assinatura devem possuir propósito separado das chaves de sessão e não devem compartilhar material criptográfico.

# 26. Task Queue
<!-- @section id="26" slug="task-queue" -->

Estados:

```text
PENDING
  ↓
PROCESSING
  ├── COMPLETED
  ├── FAILED
  ├── TIMEOUT
  ├── CANCELLED
  ├── CANCELLING
  ├── RECOVERY_PENDING
  └── PERMANENTLY_FAILED
```

Requisitos:

- retry limitado;
- timeout;
- idempotência;
- worker recovery;
- lock;
- progresso;
- cancelamento;
- tarefas órfãs;
- correlação `job_id`;
- chave de idempotência determinística.

A identidade lógica mínima de uma análise deve considerar, quando aplicável:

```text
evidence_sha256
+ operation
+ engine_version
+ ruleset_id/version/hash
+ protocol/schema version
+ parâmetros relevantes do job
```

Uma repetição do mesmo job não deve criar findings, eventos, IOCs ou artefatos duplicados silenciosamente.

## 26.1. Disponibilidade e criação do engine

O engine não é um daemon persistente no MVP. Cada job cria um novo subprocesso sandboxado. Portanto, não existe “reconexão” com uma instância do engine.

Quando a capacidade de execução estiver indisponível:

- jobs novos permanecem em `PENDING`;
- o Worker realiza verificações controladas de disponibilidade com backoff quando necessário;
- cada tentativa de processamento cria uma nova instância do engine;
- falhas de criação, execução ou sandbox devem produzir `FAILED` com código específico (`ENGINE_UNAVAILABLE`, `SANDBOX_UNAVAILABLE`, `ENGINE_PROTOCOL_ERROR`, etc.);
- a falha é auditada;
- nenhum job é silenciosamente descartado;
- não existe estado de engine compartilhado entre jobs que precise ser recuperado.

## 26.2. Decisão SQLite

**DECISÃO:** worker único no MVP.

SQLite deve usar WAL onde o ambiente suportar corretamente a política de locking.

O motor Rust não escreve diretamente no SQLite. Isso evita múltiplos escritores independentes.

## 26.3. Política de recursos do job

Para evitar transformar “limite de memória” em uma promessa sem mecanismo de enforcement, o MVP separa limites por camada:

| Recurso | Enforcement MVP | Unidade |
|---|---|---|
| Tempo total | Worker + `tokio::process::Command + timeout` | por job |
| Saída de stdout/stderr | Worker Rust + limite de protocolo | por job |
| Tamanho da linha JSONL | parser do protocolo | por mensagem |
| Eventos/itens produzidos | engine + validação do app | por job |
| CPU/RAM/PIDs | cgroup/limites do container rootless | instância do serviço; no MVP há um job pesado por vez |
| Memória estritamente por job | **não prometida no MVP** sem mecanismo de cgroup por job validado | — |

O documento não deve afirmar que um limite existe apenas porque foi colocado no objeto `limits`. Todo limite declarado como obrigatório precisa ter mecanismo de enforcement e teste correspondente.

---

## 26.4. Retry, backoff e falha permanente

**DECISÃO:** retries são limitados e somente aplicados a falhas classificadas como potencialmente transitórias. Falhas de integridade, schema incompatível, sandbox indisponível por política ou entrada inválida não devem ser repetidas indefinidamente.

Backoff inicial de referência:

```text
1s → 2s → 4s → 8s → 16s → 32s → 60s (teto)
```

O valor é configurável e deve ser registrado no job. Não é necessário afirmar que essa sequência é universalmente ótima.

Após esgotar o orçamento de tentativas, o job entra em:

```text
PERMANENTLY_FAILED
```

Esse estado é a **fila de falhas permanentes** (DLQ lógica). O job permanece pesquisável na interface, com erro final, histórico de tentativas e opção explícita de reprocessamento pelo operador com nova identidade/idempotency key quando apropriado. Reprocessamento não apaga a ocorrência original.

## 26.5. Graceful shutdown e recuperação

O processo do aplicativo deve tratar `SIGTERM` como evento de encerramento controlado. `SIGINT` deve possuir o mesmo comportamento durante execução interativa.

Fluxo:

```text
SIGTERM/SIGINT
      ↓
parar de aceitar novos jobs
      ↓
marcar Worker como DRAINING
      ↓
aguardar o job atual dentro de um prazo configurado
      ├── terminou → shutdown limpo
      └── expirou → cancelar/matar engine
                         ↓
                    persistir estado de recuperação
```

Jobs `PROCESSING` encontrados após reinício nunca podem permanecer silenciosamente nesse estado. O bootstrap deve convertê-los para `RECOVERY_PENDING`, registrar auditoria e, conforme o orçamento de retry/idempotência, reencaminhá-los para `PENDING` ou para `PERMANENTLY_FAILED`.

A aplicação não deve marcar um resultado parcial como `COMPLETED`.

## 26.6. Limite operacional de armazenamento

O sistema deve monitorar espaço livre no storage usado por evidências e artefatos.

**DECISÃO:** quando o espaço livre atingir menos de 10% do volume operacional, novos uploads de evidência devem ser rejeitados de forma controlada até que exista espaço suficiente. O limite é configurável por instalação.

O sistema deve:

- emitir alerta antes do bloqueio;
- registrar a condição em auditoria/observabilidade;
- informar a condição de forma amigável ao operador;
- preservar jobs e evidências existentes;
- nunca tentar apagar evidências automaticamente para recuperar espaço.

O limite de 90% de ocupação é uma política operacional de segurança/ disponibilidade, não uma garantia de que 10% de espaço será suficiente para qualquer workload.

# 27. Persistência
<!-- @section id="27" slug="persistencia" -->

SQLite foi escolhido por:

- operação local;
- ausência de daemon;
- backup simples;
- facilidade offline;
- tamanho apropriado para o MVP.

A arquitetura não deve assumir que SQLite é adequado para futura escala horizontal sem revisão.

## 27.1. PRAGMAs e integridade

Na inicialização da conexão, o app deve aplicar explicitamente a política de banco:

```sql
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA busy_timeout = 5000;
```

O valor de `synchronous` pode ser ajustado por benchmark somente se houver justificativa documentada de risco/benefício. O banco deve permanecer em filesystem local com locking suportado; não tratar WAL como solução para armazenamento de rede.

WAL não é tratado como mecanismo de backup. Backup deve seguir o procedimento da subseção correspondente.

Índices devem ser definidos com base em consultas reais:

- `incident_id`;
- timestamps;
- `evidence_id`;
- SHA-256;
- host;
- user_reference;
- IP;
- task status.

Paginação por cursor deve ser utilizada onde necessário.

## 27.2. Backup e restauração

"Backup simples", citado como motivo da escolha de SQLite no início da seção 27, precisa de conteúdo concreto: backup sem procedimento de restore testado não é backup, é esperança.

O que compõe um backup completo:

- banco SQLite, usando um método seguro de backup/backup online ou checkpoint controlado antes da cópia; não fazer cópia ingênua do `.db` enquanto WAL/SHM estiverem em uso;
- diretórios de storage: `evidence/`, `staging/`, `sanitized/`, `exports/` (seção 20);
- configuração de containers/Quadlet e manifesto do ambiente;
- **material criptográfico não faz parte do backup comum por padrão**. Quando houver necessidade de recuperação de chaves, deve existir procedimento separado, protegido e auditado.

Regras:

- **verificação de hash obrigatória**: todo backup gera um manifesto com SHA-256 de cada arquivo incluído; a restauração recalcula e compara antes de considerar o backup íntegro — mesma disciplina já usada para evidência (seção 19.2), aplicada ao próprio backup;
- **restore drill**: um backup nunca testado por restauração não deve ser contado como plano de recuperação válido; toda mudança de persistência que afete o formato ou procedimento de restore deve incluir um restore de teste, e o projeto deve realizar testes periódicos de restauração;
- **retenção do próprio backup**: backups de dados FORENSIC/SANITIZED herdam a mesma política de retenção da seção 21.4 — backup não é uma forma de contornar prazo de descarte;
- backup não deve rodar durante escrita ativa sem checkpoint/lock apropriado, dado o modelo de worker único (seção 26.1).

## 27.3. Caminho de migração futura — PostgreSQL (opcional)

SQLite é a decisão do MVP (seção 6) e continua sendo. Esta subseção existe só para não deixar a porta de saída indocumentada, caso o projeto precise de escala horizontal real no futuro — não é um requisito, é uma nota arquitetural.

Se a migração for necessária:

- usar SQLx mantém SQL explícito e suporte aos principais bancos sem exigir uma camada ORM; ainda assim, uma futura migração exige revisão manual de tipos específicos do SQLite e das consultas que dependem de seu comportamento;
- índices e paginação por cursor (já exigidos acima) tendem a se comportar melhor na migração do que `OFFSET` cru, exatamente pelo motivo de já terem sido exigidos aqui;
- o gargalo real que motivaria essa migração é o worker único da seção 26.1 (escritor único), não volume de dado por si só — antes de migrar de banco, vale revisitar se o problema é realmente "SQLite não aguenta" ou "a arquitetura de worker único não aguenta", que são diagnósticos diferentes;
- migração não deve ser feita preventivamente — Princípio de Necessidade Demonstrada (seção 5.1.6).

---

## 27.4. Manutenção do SQLite

**DECISÃO:** não executar `VACUUM` ou `REINDEX` automaticamente por calendário fixo.

- `VACUUM` somente em janela de manutenção e quando métricas/tamanho do banco justificarem;
- `REINDEX` somente após alterações de collation/index ou evidência técnica que justifique;
- mudanças de índices devem ser validadas com consultas reais e plano de execução;
- manutenção administrativa deve ser auditada;
- operações que possam criar picos de I/O ou uso adicional de espaço devem ser testadas no hardware de referência antes de entrar em rotina operacional.

O objetivo é evitar transformar manutenção arbitrária em risco de indisponibilidade ou consumo excessivo de armazenamento.

# 28. Containerização e isolamento
<!-- @section id="28" slug="containerizacao-e-isolamento" -->

## 28.1. Runtime

**DECISÃO:** Podman rootless é o runtime de referência.

`crun` pode ser utilizado quando disponível e validado, mas **não é uma dependência rígida da arquitetura**.

## 28.2. Arquitetura do container

O MVP terá **um único container de aplicação**, contendo dois processos Rust principais:

```text
┌────────────────────────────────────────────┐
│ dfir-app                                   │
│                                            │
│  Rust/Axum/Worker ── JSONL ── Rust Engine│
│                                            │
│  SQLite                                    │
│  Evidence / Staging / Sanitized / Exports │
│  Typst                                     │
└────────────────────────────────────────────┘
```

O objetivo continua sendo evitar múltiplos containers sem perder a separação de processo necessária para tratamento de evidências não confiáveis.

### Limite de segurança

A fronteira completa é:

```text
host
  ↓
container rootless
  ↓
processo Rust da aplicação
  ↓
sandbox bubblewrap (por job)
  ↓
processo Rust do engine
```

A linguagem comum simplifica o backend; ela não elimina a necessidade do sandbox.

Se uma ameaça ou requisito concreto justificar isolamento adicional (ex.: segundo container ou VM), isso continua possível, mas exige evidência técnica e ADR.

## 28.2.1. Build

O `Containerfile.app` deve produzir uma única imagem contendo:

- binário Rust da aplicação web;
- binário Rust `dfir-engine`;
- `bubblewrap`;
- Typst;
- `frontend/dist/app.js` e `frontend/dist/app.css`;
- ruleset do MVP;
- arquivos de migração e templates.

O build multi-stage continua recomendado para não carregar toolchains de compilação na imagem final.

O container de runtime não precisa conter `cargo`, `rustc` ou outras toolchains de compilação/desenvolvimento.

## 28.2.2. Desenvolvimento

`compose.yaml` pode existir para desenvolvimento, mas existe apenas um serviço de aplicação no MVP.

Não criar serviços independentes para banco, engine ou relatório apenas por preferência arquitetural.

## 28.3. Volumes e permissões

O container possui acesso explícito apenas a:

- `db/`;
- `evidence/`;
- `staging/`;
- `sanitized/`;
- `exports/`;
- `quarantine/`;
- `rules/yara/`.

O processo Rust deve obedecer às mesmas raízes permitidas e nunca escrever fora do staging autorizado.

Não montar indiscriminadamente:

```text
/
/etc
/home
/run/podman
```

## 28.4. Privilégio mínimo

No Linux de referência:

- rootless;
- `NoNewPrivileges=true` quando suportado;
- capabilities reduzidas;
- root filesystem somente leitura quando compatível, com diretórios graváveis montados explicitamente;
- limites de CPU/RAM/PIDs definidos no nível do container e validados por teste;
- os valores de referência devem ser escolhidos pelo benchmark do release, não por número arbitrário no documento;
- seccomp padrão do runtime salvo necessidade documentada;
- sem acesso a socket do Podman;
- nenhum segredo desnecessário no processo Rust.

## 28.5. Quadlet

Quadlet é o mecanismo de serviço persistente Linux.

Exemplo de serviço único:

```ini
[Unit]
Description=DFIR Application

[Container]
Image=localhost/dfir-app@sha256:<DIGEST>
ContainerName=dfir-app
PublishPort=127.0.0.1:8080:8080
NoNewPrivileges=true
Volume=%h/dfir/storage/db:/data/db:rw
Volume=%h/dfir/storage/evidence:/data/evidence:rw
Volume=%h/dfir/storage/staging:/data/staging:rw
Volume=%h/dfir/storage/sanitized:/data/sanitized:rw
Volume=%h/dfir/storage/exports:/data/exports:rw
Volume=%h/dfir/storage/quarantine:/data/quarantine:rw
Volume=%h/dfir/rules/yara:/data/rules/yara:ro
Secret=dfir-session-key,type=mount,target=/run/secrets/dfir-session-key,mode=0400
# Limites de CPU/RAM/PIDs devem ser definidos na instalação a partir do benchmark do release.
# Exemplo: Memory=<valor-validado> e PidsLimit=<valor-validado>.
Environment=RUST_LOG=info
Environment=DFIR_HEADLESS=true
Environment=DFIR_ENGINE_MODE=stdio

[Service]
Restart=on-failure
RestartSec=3
TimeoutStartSec=60

[Install]
WantedBy=default.target
```

O exemplo é normativo apenas como direção arquitetural; a sintaxe final deve ser validada contra a versão de Podman instalada.


# 29. Air-Gapped
<!-- @section id="29" slug="air-gapped" -->

**DECISÃO DE ENGENHARIA:** operação local sem dependência de Internet é característica do MVP; o modo Air-Gapped estrito é um perfil de implantação que remove inclusive dependências externas e assume transferência controlada de artefatos. Não é requisito textual explícito da demanda original.

## 29.1. Operação offline normal

A aplicação deve conseguir funcionar sem Internet quando operada no binding padrão `127.0.0.1`, usando seus binários, banco, regras e templates locais. Isso inclui:

- SQLite local;
- ruleset local;
- Typst local;
- assets locais;
- ausência de CDN;
- ausência de SaaS;
- ausência de API externa obrigatória.

**TLS não é necessário para o binding estritamente local em `127.0.0.1`; torna-se obrigatório quando o serviço for exposto além do loopback, conforme a seção 33.1.**

## 29.2. Perfil Air-Gapped

No perfil Air-Gapped, a máquina de análise não deve possuir dependência operacional de:

- Internet;
- SaaS;
- CDN;
- KMS/Vault externo;
- GitHub ou registry remoto durante execução;
- download automático de regras ou assinaturas.

Bundle de distribuição/atualização:

```text
offline-bundle/
├── images/
├── binaries/
├── rules/
├── migrations/
├── dataset/
├── docs/
├── licenses/
├── sbom/
├── checksums/
└── signatures/
```

## 29.3. Isolamento da máquina de análise

A máquina de análise deve ser tratada como **ambiente de investigação**, e não como ponte conveniente para a rede do incidente. Como regra operacional, deve permanecer logicamente ou fisicamente separada da rede investigada, salvo uma interface de aquisição explicitamente autorizada e controlada.

A aplicação não deve assumir que conectar-se diretamente à rede do incidente é necessário para cumprir a demanda. Coleta remota, quando necessária, pertence a uma integração específica e autorizada (ex.: Fase 4 com Velociraptor), não ao funcionamento normal do MVP.

Essa separação reduz risco de contaminação cruzada, tráfego acidental e alteração não intencional do ambiente investigado. Ela complementa — e não substitui — RBAC, auditoria, cadeia de custódia e preservação da evidência.

## 29.4. Atualização, releases e rollback

**DECISÃO:** o ciclo de atualização é controlado e explícito. O produto não realiza atualização silenciosa nem substitui uma versão em execução sem preflight, backup, validação e registro de auditoria.

```text
Release
   ↓
Manifesto + checksums + assinatura
   ↓
Preflight
   ↓
Backup verificado
   ↓
Instalação
   ↓
Migration
   ↓
Health checks
   ↓
Smoke test do engine
   ├── SUCESSO → versão ativa
   └── FALHA   → rollback controlado
```

### 29.4.1. Tipos de atualização

| Tipo | Exemplos | Política |
|---|---|---|
| Aplicação | Rust, Axum, Typst, templates | Release versionada |
| Ruleset | YARA-X e regras complementares habilitadas | Independente da aplicação quando compatível |
| Schema | migrations SQLite | Backup + preflight + migration validada |
| Bundle | pacote completo Air-Gapped | Manifesto + verificação integral |

Uma atualização de ruleset compatível não deve exigir reinstalação da aplicação.

### 29.4.2. Manifesto de release

Todo release distribuível deve possuir um manifesto contendo, no mínimo:

```text
release_version
schema_version
engine_version
app_version
ruleset_versions
container_image_digest
artifact_checksums
sbom_reference
minimum_supported_version
release_timestamp
signature_reference
```

O manifesto é a identidade do release. Em bundles de release, `latest` é proibido.

### 29.4.3. Preflight obrigatório

Antes de alterar a instalação:

- verificar integridade do artefato/bundle;
- verificar compatibilidade de versão/schema;
- verificar espaço em disco;
- verificar existência e integridade do backup;
- verificar estado da fila;
- identificar jobs `PROCESSING`;
- impedir migrations concorrentes;
- verificar que o sandbox `bubblewrap` do engine consegue ser criado e executar o perfil mínimo de isolamento (`--unshare-user`, namespaces necessários e `--disable-userns`) — falha aqui bloqueia a atualização, não apenas registra aviso;
- registrar o início da atualização na auditoria.

Jobs ativos não devem permanecer em estado ambíguo durante a troca. No MVP, a política é aguardar conclusão ou cancelar explicitamente o job antes de uma migration.

### 29.4.4. Backup antes de migration

Qualquer atualização que altere schema ou formato persistente deve criar um backup válido antes da migration. O backup só é considerado disponível quando:

1. foi criado;
2. possui manifesto;
3. possui SHA-256 dos componentes;
4. passou na verificação de integridade.

### 29.4.5. Migrations

Cada migration deve possuir:

- identificador monotônico;
- versão de origem;
- versão de destino;
- operação de aplicação;
- estratégia de falha;
- teste automatizado;
- indicação de reversibilidade.

**Não assumir que toda migration é reversível.** Quando não for, o rollback deve restaurar o backup pré-migration, em vez de criar artificialmente uma migration reversa.

### 29.4.6. Validação pós-atualização

A versão só é marcada como ativa após:

- processo iniciado;
- `/health/live` funcional;
- `/health/ready` funcional;
- leitura/escrita de banco validada;
- engine executando um smoke test determinístico;
- versão ativa registrada;
- auditoria registrada.

### 29.4.7. Rollback

Rollback possui dois níveis:

**Aplicação/container:** voltar ao artefato ou imagem anterior identificada por digest.

**Dados/schema:** restaurar o backup pré-migration quando a alteração não puder ser revertida de forma segura.

Uma imagem anterior sem backup compatível não constitui rollback completo. O procedimento deve ser testado periodicamente.

### 29.4.8. Atualização Air-Gapped

```text
máquina conectada
      ↓
criar bundle
      ↓
manifesto + checksums + assinatura
      ↓
mídia removível
      ↓
máquina isolada
      ↓
preflight
      ↓
backup
      ↓
instalação
      ↓
health + smoke test
```

Nenhuma etapa deve buscar artefatos pela Internet durante a instalação.

### 29.4.9. Atualização automática

**NÃO habilitar atualização automática no MVP.** A instalação de referência deve ser atualizada por ação explícita do operador.

A decisão prioriza previsibilidade e rastreabilidade em um sistema que trabalha com evidências forenses.

# 30. Relatórios
<!-- @section id="30" slug="relatorios" -->

## 30.1. Decisão do pipeline

**DECISÃO:** Typst será o engine principal de PDF.

Evitar o pipeline obrigatório:

```text
Markdown → Pandoc → Typst → PDF
```

para reduzir dependências e complexidade.

Usar:

```text
Domain/Report DTO
      ↓
Report renderer
  ├── Markdown export
  └── Typst export
         ↓
        PDF
```

Isso mantém Markdown como formato de interoperabilidade e Typst como formato de apresentação.

## 30.2. Segurança de templates

Dados de evidência nunca podem ser interpretados como markup Typst.

O renderer deve aplicar escaping contextual.

## 30.3. Usabilidade do analista

A interface deve priorizar análise orientada a contexto e evitar uma SPA complexa no MVP.

### Navegação

- breadcrumbs contextuais: `Incidente > Evidências > Timeline`;
- páginas de incidente devem exibir contexto e escopo atuais;
- rotas sensíveis não devem depender de estado apenas no navegador.

### Timeline

A timeline deve permitir filtros por:

- intervalo de data/hora;
- tipo de evento;
- host;
- usuário;
- origem/evidência.

A atualização pode usar HTTP/JSON e consultas paginadas por cursor. SSE pode transportar eventos unidirecionais de jobs quando necessário. Não carregar toda a timeline em memória ou no DOM sem necessidade.

### Caminho crítico

O modelo de grafo do caminho crítico deve ser a fonte de verdade. Para o MVP, a visualização pode ser renderizada como **SVG controlado pelo servidor**, sem obrigar D3.js ou Cytoscape. Bibliotecas gráficas clientes entram somente se testes de usabilidade demonstrarem ganho relevante que justifique a dependência.

A exportação vetorial em SVG é recomendada. PNG fica como extensão posterior se existir necessidade real de compatibilidade editorial.

### Relatório

Antes da exportação final, o operador deve poder gerar um **preview** sem assinar/publicar o artefato definitivo. O preview não altera a evidência nem substitui a versão final rastreável.

O sistema deve suportar pelo menos dois perfis de relatório a partir dos mesmos dados: **executivo** e **técnico**. Ambos devem preservar os mesmos identificadores, evidências e referências; apenas a apresentação e o nível de detalhe variam.

### Progresso e erros

Tarefas longas devem mostrar etapa atual e percentual quando o engine fornecer progresso confiável. Percentual desconhecido deve ser apresentado como estado indeterminado, não como número inventado.

Mensagens de erro voltadas ao usuário devem informar:

1. o que aconteceu;
2. o impacto;
3. ação recomendada;
4. identificador técnico para suporte/auditoria.

### Onboarding

O MVP terá um onboarding operacional simples baseado no Guia do Usuário e em uma lista inicial de verificação, não um tour JS obrigatório. Um tour interativo é evolução futura se houver evidência de dificuldade de adoção.

Para preservar reprodutibilidade e reduzir superfície de ataque:

- templates são arquivos confiáveis empacotados com a aplicação;
- caminhos de entrada/saída são resolvidos para raízes temporárias controladas;
- evidência não pode controlar `include`, caminhos de imagem, fontes ou outros recursos de template;
- recursos externos por URL não fazem parte do pipeline de relatório do MVP;
- o processo Typst deve ser iniciado sem shell, com timeout e diretório de trabalho controlado;
- falha de renderização resulta em falha do job de relatório, não em publicação parcial silenciosa.

---
### 30.4. Interatividade rica com TypeScript

**DECISÃO:** TypeScript é permitido no frontend para interações que exijam estado local, visualização temporal, filtros complexos ou gráficos. O código TypeScript é compilado para JavaScript estático e servido pelo `dfir-app`; Deno participa apenas do build/dev.

A Fase 0 utiliza um renderer local mínimo e sem dependências externas obrigatórias. A adoção de `vis-timeline`, D3 ou outra biblioteca de visualização somente ocorre quando a necessidade da interface demonstrar ganho real sobre a implementação existente e a dependência puder ser integrada de forma compatível com o modo offline e a política de supply chain.

### Regra HTML server-rendered × TypeScript

```text
A interação pode ser resolvida com HTML/CSS e navegação normal?
        ├── SIM → HTML server-rendered
        └── NÃO
             ↓
A interação exige estado visual local ou visualização rica?
        ├── SIM → TypeScript
        └── NÃO → HTML/CSS
```

**HTML server-rendered deve permanecer responsável por:**

- páginas e formulários da aplicação;
- navegação;
- fluxos transacionais simples;
- estados de erro e autorização;
- conteúdo que não exige estado persistente no browser.

**TypeScript deve permanecer responsável por:**

- timeline interativa;
- filtros locais de dados já autorizados;
- visualizações;
- seleção/zoom/estado visual;
- interações que não precisem de privilégios de infraestrutura.

**Transporte browser → aplicação:**

- HTTP/JSON é o transporte padrão para dados estruturados;
- SSE pode ser usado para eventos unidirecionais de jobs;
- WebSocket não faz parte do MVP.

**RESTRIÇÕES:**

- TypeScript não possui acesso direto ao SQLite, segredos ou filesystem do servidor;
- `DFIR_HTTPS=false` só é permitido por padrão em bind loopback; perfis de desenvolvimento/teste podem habilitar explicitamente `DFIR_ALLOW_INSECURE_HTTP=true`, nunca no perfil de produção. Exposições não-loopback de produção exigem HTTPS ou terminação TLS explicitamente configurada.
- o browser não é uma fronteira de autorização;
- dados recebidos pelo cliente já devem ter sido filtrados pela aplicação;
- não criar um backend Node/Bun/Deno;
- não usar APIs proprietárias de Node, Bun ou Deno no código de browser;
- não carregar bibliotecas de CDN em produção.

**CRITÉRIO DE ACEITAÇÃO:** o frontend pode ser compilado para assets estáticos e servido sem instalar Node, Bun, Deno ou Python na imagem/runtime de produção.

# 31. Assinatura de relatórios
<!-- @section id="31" slug="assinatura-de-relatorios" -->

A assinatura de relatório é um mecanismo opcional de integridade/autenticidade, não uma declaração jurídica.

**DECISÃO:** para o MVP, quando habilitada, a assinatura será **destacada**:

```text
Relatorio.pdf
Relatorio.pdf.sha256
Relatorio.pdf.sig
```

O `.sig` será uma assinatura Ed25519 do hash SHA-256 do PDF, usando uma chave de assinatura específica para relatórios.

Isso é deliberadamente diferente de “assinatura PDF incorporada”.

### Regras

- chave de relatório separada da chave de sessão;
- chave privada fora do storage de evidências;
- assinatura auditada;
- verificação obrigatória quando a funcionalidade estiver habilitada;
- não considerar o texto “Assinado digitalmente...” como evidência criptográfica;
- documentação informa exatamente qual chave/certificado foi usado e seu identificador público.

Caso um requisito futuro exija uma assinatura PDF padronizada, a implementação deverá adotar o padrão exigido naquele contexto em vez de adaptar esta assinatura destacada.

# 32. Hardening
<!-- @section id="32" slug="hardening" -->

Fluxo:

```text
Finding
  ↓
Risk
  ↓
Recommendation
  ↓
Validation
  ↓
Ansible artifact
```

Playbooks devem possuir, quando aplicável:

- dry-run/check;
- pré-condições;
- pós-condições;
- backup;
- rollback;
- alvo explícito;
- mínimo privilégio;
- auditoria.

**Nunca aplicar automaticamente em produção a partir de uma recomendação gerada pela aplicação.**

A interação com os artefatos gerados (playbooks Ansible, scripts PowerShell/Bash) é feita **exclusivamente via download pela interface web** — mesmo padrão da seção 11.3 para Velociraptor. O administrador é quem revisa, testa e executa o artefato no ambiente alvo. Aplicação do Princípio de Separação de Acesso (seção 5.1.7): a aplicação nunca contém credenciais (SSH, API) para execução remota.

---

# 33. Criptografia e gestão de chaves
<!-- @section id="33" slug="criptografia-e-gestao-de-chaves" -->

Separar:

1. dados em trânsito;
2. volume/filesystem;
3. banco;
4. campos;
5. chaves.

## 33.1. Dados em trânsito

### Modo local

No MVP, a aplicação é publicada por padrão em `127.0.0.1`. TLS não é obrigatório para o binding estritamente local, pois não existe trânsito de rede entre hosts; isso não implica que a sessão local esteja livre de outros riscos de máquina comprometida.

Nesse modo, o tráfego permanece no host e não é tratado como transporte de rede remota.

### Modo LAN/rede

Se a aplicação for exposta além do loopback:

- TLS é obrigatório;
- preferir TLS 1.3;
- desabilitar TLS legado e suites fracas;
- certificados devem possuir origem e validade documentadas;
- renovação deve ser definida;
- HSTS deve existir quando a aplicação estiver efetivamente em HTTPS.

A aplicação não deve assumir que "HTTP interno" é seguro se o serviço estiver acessível pela LAN.

## 33.2. Dados em repouso — decisão do MVP

**DECISÃO:** não introduzir SQLCipher como requisito do MVP.

Na instalação de referência Linux, a proteção de FORENSIC deve ocorrer preferencialmente no nível do **volume/filesystem criptografado** que contém:

- SQLite;
- evidências;
- staging;
- sanitizados;
- exports.

Exemplo de referência: LUKS2 ou mecanismo corporativo equivalente.

Motivos:

- protege o conjunto inteiro;
- não introduz fork/variante adicional de SQLite;
- simplifica backup/restauração;
- reduz complexidade da camada de acesso SQLite;
- funciona para arquivos que não estão no banco.

### Criptografia seletiva de campos

Pode ser adicionada para campos que exigem proteção adicional.

Quando implementada:

- usar AEAD moderno, como AES-256-GCM;
- chave não pode ficar no mesmo banco em claro;
- registrar versão do esquema de criptografia;
- planejar rotação e recuperação.

## 33.3. Gestão de chaves

No MVP:

- secrets de sessão → `podman secret` ou mecanismo de segredo equivalente;
- chave de assinatura da chain → separada;
- chave de assinatura de relatórios → separada;
- nenhum segredo no Git;
- nenhum segredo na imagem;
- nenhum segredo no `.env` versionado.

Vault/KMS fica como extensão futura.

### 33.3.1. Higiene de memória de segredos

**DECISÃO:** qualquer material de chave que precise existir em memória do processo Rust (chave de assinatura carregada para uso, secret de sessão decodificado) deve usar um tipo que zera a memória ao sair de escopo, via `zeroize`. [37]

Isso não substitui os controles de localização da subseção 33.3 (onde o segredo fica armazenado); reduz a janela em que uma chave permanece legível em RAM após o uso — por exemplo, em um core dump, ou por outro processo com acesso à mesma memória. Não é proteção contra ataques de microarquitetura (ex.: classes Spectre/Meltdown); isso é uma limitação documentada da própria técnica, não uma falha de configuração do projeto.

## 33.4. Air-Gapped

O projeto deve operar sem KMS/Vault externo.

A recuperação de chaves deve estar documentada e depender de mecanismo local controlado pelo administrador.

O pacote de backup não deve conter chave privada em texto puro junto dos dados sem uma proteção equivalente ou superior.

> **Nota:** proteção criptográfica não substitui controle de acesso, backup, retenção ou auditoria.

# 34. Observabilidade e auditoria
<!-- @section id="34" slug="observabilidade-e-auditoria" -->

Logs estruturados, implementados com `tracing`/`tracing-subscriber` (seção 6.5):

```json
{
  "timestamp": "2026-08-13T00:00:00Z",
  "level": "INFO",
  "event": "evidence_ingested",
  "incident_id": "inc_...",
  "evidence_id": "ev_...",
  "actor_id": "usr_...",
  "trace_id": "trace_..."
}
```

Nunca registrar diretamente:

- senhas;
- tokens;
- chaves privadas;
- secrets;
- conteúdo sensível sem necessidade.

Auditar:

- login/logout;
- acesso a evidência;
- exportação;
- alteração NIST;
- execução de scanner;
- geração/download de artefato de hardening e importação de resultado externo;
- importação/exportação `.dfirpkg`;
- chain of custody;
- atualização offline;
- alterações de configuração;
- descarte de dados;
- criação/alteração de permissões.

## 34.1. Health checks

Distinguir:

```text
liveness = processo responde
readiness = DB + storage + engine subprocessável estão operacionais
```

Endpoints internos sugeridos:

```text
GET /health/live
GET /health/ready
```

Não expor estes endpoints publicamente fora do binding/controle administrativo definido para a instalação.

Resposta de readiness:

```json
{
  "status": "ready",
  "database": "ok",
  "engine": "ok",
  "evidence_storage": "ok",
  "disk_free_pct": 62
}
```

Se o engine não estiver disponível:

```json
{
  "status": "not_ready",
  "engine": "unavailable"
}
```

Isso não deve apagar jobs. A fila deve permanecer auditável.

## 34.2. Métricas operacionais mínimas

- profundidade da fila;
- idade do job mais antigo;
- falhas/timeouts do engine;
- espaço livre no storage;
- tempo médio de processamento;
- número de jobs por estado;
- último sucesso do engine.

Prometheus/Grafana não são requisitos do MVP.

## 34.3. Controle de armazenamento

Além das métricas operacionais, o app deve distinguir:

- espaço livre total do volume;
- espaço livre reservado ao storage de evidências;
- condição de aviso;
- condição de bloqueio de novos uploads.

O bloqueio é definido na seção 26.6. A interface deve informar claramente quando o usuário estiver impedido de importar por falta de espaço, sem mascarar o erro como falha de análise.

## 34.4. Auditoria tamper-evident

A tabela `AuditEvent` não é tratada como imutável apenas por convenção.

No caminho normal da aplicação:

- `AuditEvent` é append-only;
- não existe rota de negócio para `UPDATE` ou `DELETE` de eventos;
- o ator é derivado da sessão/autorização, nunca aceito como identidade livre do cliente;
- campos de auditoria são normalizados e canonizados antes do hash;
- alterações administrativas sensíveis ao próprio sistema também geram eventos de auditoria.

A cadeia de hashes da seção 22.4 deve ser verificável com uma operação administrativa:

```text
verifyAuditChain()
```

Alterações detectadas devem gerar um estado de integridade:

```text
VALID
BROKEN
INCOMPLETE
```

A ocorrência de `BROKEN` não deve ser apagada pelo próprio processo de auditoria; deve ser preservada como incidente de integridade.

---
# 35. Testes
<!-- @section id="35" slug="testes" -->

Camadas:

```text
Unit
Integration
Security
End-to-End
Performance
Forensic Integrity
Compatibility
```

## 35.1. Casos mínimos

### Evidência

- hash;
- arquivo vazio;
- arquivo grande;
- duplicado;
- arquivo alterado após ingestão;
- path traversal;
- archive bomb.

### Parsing

- EVTX válido;
- EVTX corrompido;
- campo ausente;
- evento desconhecido;
- timestamp inválido;
- erro não silencioso.

### YARA-X

- match;
- no match;
- regra inválida;
- cache invalidado após mudança de ruleset.

### Queue

- crash do worker;
- retry;
- timeout;
- concorrência;
- duplicação;
- cancelamento.

### Segurança

- XSS;
- command injection;
- path traversal;
- Zip Slip;
- symlink abuse;
- upload malicioso;
- autorização insuficiente;
- acesso cruzado entre incidentes;
- permissão de destruição de evidência;
- permissão de alteração de acesso;
- sessão;
- CSRF em rotas de mutação (seção 24).

### Relatórios

- injeção de markup Typst;
- tentativa de incluir caminho externo;
- tentativa de acessar arquivo fora da raiz de renderização;
- timeout de renderização;
- ausência de publicação parcial após falha.

### Chain of Custody

- cadeia válida;
- alteração;
- remoção;
- reordenação;
- hash incorreto;
- assinatura inválida.

### Engine

- CLI JSONL válido;
- schema/version incompatível;
- `request_id`/`job_id` incorretos;
- linha JSONL acima do limite;
- falha controlada;
- timeout real via `tokio::process::Command`;
- `maxBuffer`/limite de saída;
- engine sem sandbox → job rejeitado;
- sandbox sem rede;
- tentativa de acesso a mount não concedido;
- ausência de secrets no ambiente do engine;
- limite de eventos/resultado;
- limite de memória conforme mecanismo efetivamente implementado.

---

### Smoke tests

Um conjunto curto de smoke tests deve rodar em cada build verificável, cobrindo no mínimo:

- upload de evidência;
- parsing de um fixture EVTX;
- execução de um ruleset YARA-X mínimo;
- geração de um relatório simples.

### Carga e concorrência

Devem existir cenários controlados com:

- EVTX acima de 500 MB;
- múltiplos uploads HTTP concorrentes;
- tentativa de iniciar mais de um job pesado enquanto o Worker já está `PROCESSING`;
- armazenamento próximo do limite operacional.

O objetivo é validar degradação controlada, fila, backpressure e limites. O teste deve confirmar que o Worker não executa dois jobs pesados simultaneamente no MVP.

### Teste dinâmico opcional

OWASP ZAP pode ser usado em ambiente de teste para uma varredura baseline da interface HTTP. É ferramenta de teste, não dependência do produto, e não bloqueia o MVP quando a superfície já estiver coberta pelos testes específicos obrigatórios. [22]

### Dataset real/sanitizado

Dados reais somente podem ser usados com autorização e sanitização adequada. O dataset sintético BEC continua sendo a base determinística do MVP. Benchmarks adicionais com dados reais autorizados são úteis, mas não são requisito obrigatório da demonstração.

### Proveniência dos fixtures de teste

Distinto do dataset de demonstração (seção 42): os fixtures usados por `cargo test`/CI (EVTX, PCAP, registry hive, amostras YARA-X) devem ter origem registrada, não apenas existir no repositório.

Para cada fixture:

- origem (gerado sinteticamente pelo projeto, ou de repositório público de amostras forenses);
- licença/termos de uso da fonte, quando aplicável;
- SHA-256, para reprodutibilidade dos testes que dependem dele (mesma disciplina da seção 19.2 e do backup, seção 27.2);
- confirmação de que não contém dado real/PII, mesmo quando a fonte for pública.

Fixtures não devem ser adicionados "porque foram encontrados prontos" sem essa verificação — isso vale tanto para amostras geradas pelo projeto quanto para amostras importadas de terceiros.

---

# 36. Performance
<!-- @section id="36" slug="performance" -->

## 36.1. Regras

Valores de benchmark são metas de engenharia.

Todo benchmark deve registrar:

- hardware;
- sistema operacional;
- versão do app;
- versão do engine;
- versão do ruleset;
- dataset;
- tamanho;
- número de eventos;
- threads;
- limites de memória/CPU;
- método;
- resultado.

## 36.2. Metas iniciais

| Métrica | Meta inicial | Limite aceitável |
|---|---:|---:|
| Inicialização do app | < 3 s | < 10 s |
| RAM do app em repouso | < 100 MB | < 200 MB |
| EVTX 200 MB | < 300 MB RAM | < 500 MB RAM |
| Relatório simples | < 10 s | < 30 s |
| Scan YARA-X controlado | < 5 s | < 15 s |

Esses valores são metas iniciais e não garantias.

## 36.3. Hardware de referência

Separar **runtime mínimo** de **ambiente de compilação**.

### Runtime mínimo de referência

- 4 GB RAM;
- CPU x86_64 de baixo consumo;
- Linux minimal;
- armazenamento local suficiente.

O objetivo é que uma build release previamente compilada consiga operar dentro desse piso para o MVP.

### Desenvolvimento/compilação

8 GB RAM ou mais é recomendado para compilar Rust + YARA-X + Polars com conforto.

Em hardware menor:

- `CARGO_BUILD_JOBS=1`;
- preferir `cargo check` durante iteração;
- evitar `target/` em tmpfs salvo benchmark que prove benefício;
- não transformar RAM disk em dependência.

## 36.4. Testes sob restrição

Pelo menos um benchmark por release deve reproduzir o ambiente de runtime mínimo, por exemplo:

```text
engine container
memory limit = 512m
```

O objetivo é detectar consumo excessivo antes da apresentação.

## 36.5. Performance externa

Números como “benchmarks absolutos” ou “5–10x” podem ser citados apenas como benchmark externo de referência e nunca como propriedade garantida desta aplicação.

## 36.6. Pipeline de benchmark

**DECISÃO:** os benchmarks devem ser reproduzíveis por comandos versionados. As ferramentas são de desenvolvimento, não dependências do produto.

Perfil mínimo recomendado:

```text
hyperfine       → comandos/CLI/processos completos
cargo bench     → microbenchmarks Rust realmente necessários
just perf       → ponto de entrada unificado
```

`hyperfine` mede tempo de execução de comandos/CLI/processos completos de ponta a ponta e não é dependência do produto. [21]

`just perf` deve produzir resultados estruturados, preferencialmente em JSON, e registrar:

- release/version;
- commit ou identificador do source;
- hardware/OS;
- dataset e SHA-256;
- engine/ruleset;
- tempo de parsing EVTX;
- tempo de scan YARA-X;
- tempo de geração de relatório;
- pico de RAM;
- tempo de inicialização;
- falhas/timeouts;
- parâmetros relevantes.

As metas da seção 36.2 são **metas de engenharia**. Uma release só deve publicar números de performance como resultados medidos quando o ambiente e o método estiverem registrados.

# 37. Estrutura do projeto
<!-- @section id="37" slug="estrutura-do-projeto" -->

```text
projeto-dfir-nist/
├── src/
│   ├── main.rs
│   ├── web/
│   ├── domain/
│   ├── application/
│   ├── infrastructure/
│   ├── persistence/
│   │   ├── db.rs
│   │   ├── repositories.rs
│   │   └── audit.rs
│   ├── security/
│   ├── reports/
│   └── jobs/
├── engine/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── parser/
│   │   ├── scanner/
│   │   ├── extractor/
│   │   ├── correlation/
│   │   ├── processor/
│   │   └── protocol/
│   └── tests/
├── templates/
│   ├── html/
│   ├── markdown/
│   └── typst/
├── frontend/
│   ├── deno.json
│   ├── src/
│   │   ├── app.ts
│   │   └── app.css
│   └── dist/
│       ├── app.js
│       └── app.css
├── rules/
│   └── yara/
├── collectors/
│   └── velociraptor/          # Fase 4
├── ansible/
├── releases/
│   └── manifests/
├── storage/
│   ├── db/
│   ├── evidence/
│   ├── staging/
│   ├── sanitized/
│   ├── exports/
│   └── quarantine/
├── containers/
│   └── Containerfile.app
├── quadlet/
│   └── dfir-app.container
├── scripts/
│   ├── setup.sh
│   └── setup.ps1
├── tests/
│   ├── unit/
│   ├── integration/
│   ├── security/
│   ├── performance/
│   └── fixtures/
├── docs/
│   ├── adrs/
│   └── rtm.md
├── justfile
├── .envrc
├── Cargo.toml
├── Cargo.lock
├── compose.yaml               # desenvolvimento, opcional e tool-agnostic
├── LICENSE
├── LICENSES_THIRD_PARTY.md
├── SBOM.json
├── CHANGELOG.md
├── CONTRIBUTING.md             # quando houver contribuições externas
└── README.md
```

O workspace Rust deve conter pelo menos `dfir-app` e `dfir-engine`. O frontend possui TypeScript como toolchain de build/dev e produz somente assets estáticos para o runtime. Deno é ferramenta de build/dev; CPython e Shell são ferramentas de engenharia/QA e não pertencem ao runtime de produção.

## 37.1. Legibilidade, manutenibilidade e clareza do código
<!-- @section id="37.1" slug="legibilidade-manutenibilidade-e-clareza-do-codigo" -->

**OBRIGATÓRIO:** o código deve ser escrito para permanecer compreensível por outro desenvolvedor ou por uma LLM que precise mantê-lo posteriormente. Legibilidade é requisito de engenharia e não apenas preferência de estilo.

### 37.1.1. Princípios

A implementação deve priorizar, nesta ordem:

```text
clareza
↓
correção e segurança
↓
rastreabilidade
↓
manutenibilidade
↓
concisão
```

Código não deve ser tornado mais curto quando isso aumentar a carga cognitiva, esconder invariantes ou dificultar a revisão de segurança. Não impor limites artificiais de linhas para funções, módulos ou arquivos; tamanho é aceitável quando a coesão e a leitura permanecerem claras.

### 37.1.2. Nomenclatura

- nomes devem representar intenção e domínio;
- evitar abreviações não padronizadas;
- evitar nomes genéricos (`data`, `item`, `thing`, `manager`, `process`) quando um nome de domínio mais preciso estiver disponível;
- funções devem expressar a operação realizada;
- tipos devem representar conceitos do domínio ou contratos técnicos;
- booleanos devem possuir nomes que deixem claro o estado ou condição;
- identificadores usados em segurança, auditoria, evidência, jobs e protocolo devem permanecer semanticamente estáveis.

Mudanças de nomenclatura que alterem contratos persistidos, JSONL, schema, API ou RTM exigem atualização explícita dos artefatos afetados.

### 37.1.3. Funções, módulos e coesão

Cada função e módulo deve possuir uma responsabilidade compreensível e uma fronteira de dependências identificável. Evitar tanto funções monolíticas quanto fragmentação artificial em helpers triviais sem ganho real de compreensão ou teste.

A estrutura de módulos deve respeitar as fronteiras arquiteturais já definidas: domínio não depende de infraestrutura concreta; `dfir-app` supervisiona o engine; o engine não acessa persistência de negócio.

### 37.1.4. Abstração sob necessidade demonstrada

**OBRIGATÓRIO:** não criar abstrações especulativas. Traits, interfaces, factories, wrappers, registries, providers ou camadas adicionais somente devem existir quando resolverem um problema concreto de substituição, isolamento, teste, segurança, composição ou requisito do domínio.

A LLM implementadora não deve introduzir uma cadeia de camadas apenas para aparentar aderência a Clean Architecture, SOLID ou DDD. Uma abstração menor e explícita é preferível a uma hierarquia sem necessidade demonstrada.

### 37.1.5. Comentários e documentação

Comentários devem explicar principalmente **por que** uma decisão ou workaround existe, e não repetir literalmente o código.

Comentários ou documentação são especialmente recomendados quando houver:

- invariantes de segurança;
- comportamento forense não óbvio;
- workaround de dependência;
- limitação conhecida de plataforma;
- compatibilidade;
- motivo de uma otimização que sacrifique legibilidade;
- regra necessária para preservar integridade, auditoria ou cadeia de processamento.

Não documentar cada operação trivial apenas para aumentar a quantidade de comentários. A complexidade da documentação deve ser proporcional ao risco e à relevância da interface.

### 37.1.6. Erros, invariantes e controle de fluxo

- não utilizar `unwrap()` ou `expect()` em caminhos de entrada ou processamento não confiável sem justificativa verificável;
- erros esperados devem ser representados por `Result` e tipos de erro explícitos;
- não ocultar falhas com `continue`, valores padrão ou conversões silenciosas quando a falha for relevante para integridade ou análise;
- condições de segurança devem permanecer visíveis no fluxo de controle;
- mensagens de erro devem ser sanitizadas quando atravessarem fronteiras de usuário ou auditoria;
- invariantes críticas devem ser verificadas no ponto em que deixam de ser garantidas pela fronteira anterior.

Em particular, uma falha de um registro do parser não pode desaparecer silenciosamente, conforme a regra de integridade do parsing.

### 37.1.7. Código morto e escopo

Não manter código morto, imports não utilizados, módulos vazios, comentários de código obsoleto, TODOs abandonados ou esqueletos de funcionalidades futuras apenas para antecipar fases posteriores.

Funcionalidades de Fase 2–5 não devem introduzir estruturas de produção na Fase 0/1 sem necessidade demonstrada e sem registro da alteração.

### 37.1.8. Código sensível e rastreabilidade

Operações que alterem segurança, acesso, evidência, auditoria, jobs, configuração ou estado persistente devem permanecer fáceis de localizar e revisar. Não esconder efeitos sensíveis em macros, helpers genéricos ou callbacks cuja semântica não seja evidente.

O código deve permitir rastrear, de forma razoavelmente direta:

```text
entrada
↓
validação
↓
autorização
↓
operação
↓
auditoria/resultado
```

### 37.1.9. Regras específicas para LLMs

Antes de alterar código existente, a LLM deve preservar o comportamento não relacionado à tarefa. Não é permitido:

- refatorar módulos sem necessidade para a tarefa;
- substituir bibliotecas por preferência pessoal;
- renomear grandes conjuntos de símbolos sem benefício demonstrado;
- duplicar lógica existente quando uma função ou módulo apropriado já existir;
- criar abstrações para uma necessidade futura não confirmada;
- implementar funcionalidades de fases futuras por antecipação;
- apagar validações, testes ou auditoria para simplificar uma implementação;
- alterar contratos persistidos ou de integração sem atualizar os artefatos dependentes;
- introduzir comentários ou documentação que afirmem garantias que o código não implementa.

Quando a tarefa exigir uma refatoração estrutural, a LLM deve indicar o motivo, o ganho esperado e os testes que demonstram a preservação do comportamento.

### 37.1.10. Critério de aceitação

Uma implementação não atende ao requisito de legibilidade quando sua compreensão exigir conhecimento implícito que poderia ser expresso por nomes, tipos, estrutura, testes ou documentação objetiva. Revisões devem conseguir responder, sem investigação excessivamente indireta:

1. onde está a regra;
2. quais entradas ela aceita;
3. quais invariantes protege;
4. quais efeitos colaterais produz;
5. como seus erros são tratados;
6. como seu comportamento é testado.

## 37.2. Responsabilidades dos diretórios e módulos
<!-- @section id="37.2" slug="responsabilidades-dos-diretorios-e-modulos" -->

**OBRIGATÓRIO:** a estrutura de diretórios deve expressar responsabilidades arquiteturais. A árvore apresentada na seção 37 é uma **estrutura de referência**: os nomes e a divisão dos arquivos concretos podem variar quando houver justificativa técnica, mas nenhuma alteração pode violar as fronteiras abaixo.

### 37.2.1. Aplicação `dfir-app`

| Área | Responsabilidade | Não deve conter |
|---|---|---|
| `src/domain/` | Entidades, value objects, invariantes e regras de negócio independentes de infraestrutura | Axum, SQLx, filesystem, subprocessos, detalhes HTTP |
| `src/application/` | Casos de uso, orquestração, políticas de aplicação e contratos necessários à execução dos casos de uso | Detalhes concretos de SQLite, HTTP ou processo do engine quando houver abstração apropriada |
| `src/infrastructure/` | Implementações concretas de persistência, filesystem, processos, integração local, configuração e demais recursos externos | Regras de negócio que deveriam pertencer ao domínio |
| `src/persistence/` | Mapeamento e operações de persistência SQLx/SQLite, repositories finos, migrations e transações | Regras de autorização ou lógica de negócio arbitrária; ORM obrigatório |
| `src/security/` | Autenticação, sessão, hashing de senha, CSRF, rate limiting, autorização e controles criptográficos da aplicação | Regras de negócio de incidentes/evidências não relacionadas à segurança da aplicação |
| `src/web/` | Rotas, handlers HTTP, extração de entrada, respostas, middleware e entrega de assets frontend | Acesso direto ao SQLite ou regras de domínio embutidas em handlers |
| `frontend/src/` | Código TypeScript de browser, timeline e interações visuais | Acesso ao banco, segredos, regras de autorização ou APIs de infraestrutura |
| `frontend/dist/` | JavaScript estático compilado para distribuição | Código TypeScript não compilado ou dependências de runtime externo |
| `scripts/`, `tests/` | Python/Shell para QA, fixtures, benchmarks e automação | Dependência do runtime de produção ou acesso privilegiado por padrão |
| `src/reports/` | Modelos e preparação de dados para relatórios e futuras renderizações | Regras de análise forense que pertençam ao domínio/aplicação |
| `src/jobs/` | Worker, estados de job, supervisão, timeout, cancelamento, retry, recovery e comunicação com o engine | Parsing forense ou persistência direta pelo engine |

As áreas podem ser subdivididas em módulos menores quando a coesão exigir. Não é obrigatório criar um arquivo por entidade, caso de uso, endpoint ou interface.

### 37.2.2. Engine `dfir-engine`

| Área | Responsabilidade | Não deve conter |
|---|---|---|
| `engine/src/parser/` | Parsing de formatos suportados e representação intermediária dos registros | Autenticação, RBAC ou persistência de negócio |
| `engine/src/scanner/` | Detecção baseada em engines/rulesets, incluindo YARA-X quando implementado | Políticas de usuário ou auditoria da aplicação |
| `engine/src/extractor/` | Extração determinística de IOCs e outros atributos derivados | Decisões de negócio ou acesso ao banco da aplicação |
| `engine/src/correlation/` | Correlação determinística de resultados e relações técnicas | Alteração de estado de negócio |
| `engine/src/processor/` | Orquestração interna do processamento, limites, normalização e preparação de resultados | HTTP, sessão ou SQLite |
| `engine/src/protocol/` | Contratos JSONL, validação de mensagens e serialização/deserialização | Regras de autorização da aplicação |

O engine permanece um processo especializado e não conhece autenticação, RBAC, sessão, NIST, auditoria de usuário ou persistência de negócio, conforme a fronteira arquitetural já definida.

### 37.2.3. Diretórios de suporte

- `templates/`: somente templates e recursos necessários à apresentação/renderização; não conter regras de domínio.
- `frontend/`: fonte TypeScript/CSS e artefatos estáticos versionados; sem lógica de domínio, sem CDN obrigatório e sem runtime JS de servidor.
- `rules/`: rulesets e metadados de proveniência, nunca credenciais ou configuração de runtime.
- `collectors/`: somente componentes de coleta posterior explicitamente autorizados; itens de Fase 4 não devem bloquear o MVP.
- `ansible/`: artefatos de hardening gerados ou versionados, sem aplicação automática em produção.
- `releases/manifests/`: manifests, checksums, digests, versões efetivas e metadados de release.
- `storage/`: dados operacionais e evidências; código não deve depender de nomes de arquivos controlados pelo usuário.
- `tests/`: testes que cruzem fronteiras de módulo ou precisem de fixtures compartilhadas; testes unitários próximos ao módulo continuam permitidos.
- `docs/adrs/`: decisões arquiteturais e suas justificativas; não substituir a documentação normativa principal.

### 37.2.4. Regra de dependência

A organização deve preservar a direção lógica: 

```text
presentation / web
        ↓
application
        ↓
domain
```

Implementações de infraestrutura podem satisfazer contratos necessários à aplicação/domínio, mas o domínio não deve depender de detalhes concretos de infraestrutura. A presença de uma pasta `infrastructure/` não autoriza mover regras de negócio para ela.

Dependências compartilhadas devem ser justificadas pela responsabilidade real do módulo. Não criar um módulo `common`, `utils`, `helpers` ou equivalente como depósito genérico para código sem dono arquitetural definido.

### 37.2.5. Arquivos concretos e flexibilidade

A especificação **não fixa um nome de arquivo para cada responsabilidade**. Um módulo pode permanecer em um único arquivo quando pequeno e coeso, ou ser dividido quando o crescimento justificar. A divisão deve preservar nomes e fronteiras compreensíveis.

A LLM implementadora deve preferir a menor estrutura que mantenha: 

1. responsabilidade única compreensível;
2. dependências visíveis;
3. testes localizáveis;
4. baixo acoplamento;
5. rastreabilidade de segurança e auditoria.

### 37.2.6. Critério de alteração estrutural

Mover responsabilidade entre diretórios, introduzir uma nova camada arquitetural ou criar um novo módulo compartilhado exige: 

- motivo técnico concreto;
- preservação das fronteiras arquiteturais;
- atualização dos testes afetados;
- atualização da documentação quando a estrutura de referência deixar de representar o projeto;
- ADR quando a alteração constituir nova decisão arquitetural.

Uma alteração estrutural não é justificativa para refatorar código não relacionado à tarefa.

# 38. Roadmap
<!-- @section id="38" slug="roadmap" -->

## Fase 0 — Fundamentos

- arquitetura;
- SQLite;
- migrations;
- autenticação;
- RBAC;
- auditoria;
- storage;
- Podman rootless;
- engine protocol.

## Fase 1 — MVP DFIR

- evidência;
- SHA-256;
- chain of custody;
- EVTX;
- timeline;
- YARA-X;
- IOCs;
- múltiplos incidentes;
- `.dfirpkg`;
- relatórios.

## Fase 2 — NIST e análise

- Current Profile;
- Target Profile;
- Gap Analysis;
- Tiers contextualizados;
- análise de caminho crítico;
- matriz de impacto;
- ATT&CK.

## Fase 3 — Entregáveis e demonstração

- PDF Typst;
- Markdown/JSON;
- dataset BEC;
- assinatura de relatório quando habilitada;
- modo offline;
- ciclo de atualização e rollback;
- dashboard executivo **somente se houver tempo e valor demonstrado; não é requisito da demanda**.

## Fase 4 — Coleta especializada e automação posterior

- Velociraptor (geração de artefato + importação manual, seção 11.3 — sem acesso direto à API);
- coleta agentless, somente após caso de uso definido;
- geração de artefatos Ansible;
- parsing de PCAP;
- ClamAV opcional;
- geração/validação de Sigma.

## Fase 5 — Extensões

- Vault/KMS;
- M365/Entra ID;
- Google Workspace;
- memory forensics;
- LLM local opcional.

**Nenhuma funcionalidade classificada como Fase 4/5 pode bloquear o MVP. Os critérios de aceitação dessas fases só são cobrados quando a respectiva fase for implementada.**

---

# 39. Critérios de aceitação
<!-- @section id="39" slug="criterios-de-aceitacao" -->

| ID | Fase | Requisito | Critério |
|---|---|---|---|
| RF-001 | MVP | Upload | Arquivo válido recebe UUID e Evidence. |
| RF-002 | MVP | Integridade | SHA-256 antes da análise. |
| RF-003 | MVP | Preservação | Original não alterado. |
| RF-004 | MVP | Custódia | Cadeia verificável. |
| RF-005 | MVP | EVTX | Eventos normalizados. |
| RF-006 | MVP | Erro de parsing | Erro registrado, nunca descartado silenciosamente. |
| RF-007 | MVP | Timeline | Ordenação e referência à evidência. |
| RF-008 | MVP | YARA-X | Resultado estruturado e versionado. |
| RF-009 | MVP | NIST | Current/Target Profiles persistidos. |
| RF-010 | MVP | Gap | Gap reproduzível a partir dos dados. |
| RF-011 | MVP | ATT&CK | Mapping possui evidência e confiança. |
| RF-012 | MVP | Hardening | Recomendação pode gerar artefato Ansible quando aplicável. |
| RF-013 | MVP | Entregáveis | Cinco documentos cobrem os seis requisitos de conteúdo da demanda. |
| RF-014 | MVP | Offline | MVP executa sem Internet. |
| RF-015 | MVP | Auditoria | Ações sensíveis auditadas. |
| RF-016 | MVP | Segurança | Testes de XSS/injection/traversal passam. |
| RF-017 | MVP | Demo | Dataset determinístico e sanitizado. |
| RF-018 | MVP | Caminho crítico | Eventos relacionados podem ser ligados com evidência/confiança. |
| RF-019 | MVP | IOCs | IOCs extraídos com origem. |
| RF-020 | MVP | Impacto | Matriz de impacto com confiança. |
| RF-021 | F4 | Sigma | Regra exportada e validada quando possível. |
| RF-022 | MVP | Imediato/Estratégico | Recomendações separadas. |
| RF-023 | F4 | Velociraptor | Fase 4, não bloqueante. |
| RF-024 | MVP | SBOM/licenças | Documentação gerada com versões reais. |
| RF-025 | MVP | `.dfirpkg` | Importação valida manifest/checksums antes de persistir. |
| RF-026 | MVP | Atualização offline | Bundle verificado antes da instalação. |
| RF-027 | MVP | Protocolo do engine | Produção usa subprocesso + JSONL; nenhum endpoint TCP/Unix Socket do engine é exposto. |
| RF-028 | MVP | Engine separado por processo | Engine não acessa SQLite diretamente e opera sob limites explícitos de tempo/tamanho. |
| RF-029 | MVP | Auditoria | Cadeia de AuditEvent pode ser verificada e acusa adulteração. |
| RF-030 | MVP | Criptografia | Instalação de referência documenta proteção do volume e modo de exposição da aplicação. |
| RF-031 | MVP | Supply chain | Release possui `Cargo.lock`, SBOM e imagens identificadas por digest. |
| RF-032 | MVP | Atualização controlada | Release verifica manifesto/integridade, executa preflight, preserva backup antes de migration e realiza health/smoke checks antes de marcar a versão como ativa. |
| RF-033 | MVP | Isolamento do engine | Processo Rust possui limites de job, ambiente reduzido, staging por job e encerramento supervisionado sem acesso direto à persistência de negócio. |
| RF-034 | MVP | Fontes de evidência | A aplicação permite importar/preservar evidências de diferentes fontes citadas na demanda, registra a origem e informa explicitamente quando a análise especializada do formato ainda não está implementada. |
| RF-035 | MVP | Contrato JSONL | Mensagens possuem versão, tipo, `request_id`/`job_id`, limites e tratamento explícito de erro; caminhos físicos do storage não atravessam o contrato do engine. |
| RF-036 | MVP | Isolamento fechado | Falha na criação do sandbox impede a execução do engine; não existe fallback silencioso para execução sem sandbox. |
| RF-037 | MVP | Recursos | Timeout e limites de saída possuem enforcement testado; CPU/RAM/PIDs são limitados pelo mecanismo efetivamente configurado, sem alegar enforcement por job não implementado. |
| RF-038 | MVP | Autorização por incidente | Operações sensíveis exigem papel global compatível, acesso ao incidente e permissão de domínio correspondente. |
| RF-039 | MVP | Idempotência | Reprocessamento do mesmo job não cria artefatos de análise duplicados silenciosamente. |
| RF-040 | MVP | Relatórios | Dados de evidência não conseguem alterar a estrutura de template nem acessar recursos arbitrários durante a renderização. |
| RF-041 | MVP | Retry/DLQ | Falhas transitórias usam retry limitado com backoff; após esgotamento o job fica `PERMANENTLY_FAILED` e permanece inspecionável. |
| RF-042 | MVP | Shutdown/Recovery | `SIGTERM`/`SIGINT` interrompem a aceitação de novos jobs e jobs `PROCESSING` são recuperados explicitamente após reinício. |
| RF-043 | MVP | Armazenamento | Abaixo do limite configurado de espaço livre, novos uploads são rejeitados e a condição é registrada sem apagar evidências automaticamente. |
| RF-044 | MVP | Validação | Entradas HTTP e envelope JSONL inválidos são rejeitados antes de alcançar o domínio/engine; o Rust valida novamente a própria fronteira. |
| RF-045 | MVP | Benchmark | `just perf` reproduz o conjunto mínimo de benchmarks e grava metadados suficientes para reproduzir os resultados. |
| RF-046 | MVP | UX de análise | Timeline suporta filtros mínimos, tarefas longas exibem estado/progresso quando disponível e relatórios possuem preview e perfil executivo/técnico. |
| RF-047 | MVP | Versionamento | Todo release usa versões estáveis mais recentes adotadas conforme a política da seção 6.6, registra versões exatas, lockfiles/checksums/digests e não utiliza identificadores flutuantes como `latest`. |

---

# 40. Definition of Done
<!-- @section id="40" slug="definition-of-done" -->

Uma funcionalidade só será considerada concluída quando:

```text
[ ] Requisito identificado
[ ] Caso de uso definido
[ ] Implementação concluída
[ ] Tratamento de erro
[ ] Auditoria/logs adequados
[ ] Teste unitário
[ ] Teste de integração quando aplicável
[ ] Teste de segurança quando aplicável
[ ] Critério de aceitação validado
[ ] Documentação atualizada
[ ] Sem regressões conhecidas
[ ] cargo audit executado se dependência Rust foi adicionada ou atualizada (Apêndice C.6)
[ ] Atualização/rollback testados quando a mudança altera release, schema, ruleset ou bundle
[ ] Backup e restore verificados quando a mudança altera persistência
[ ] Contrato de integração/JSONL versionado e testado quando alterado
[ ] Autorização/RBAC testada quando uma permissão ou fluxo sensível é alterado
[ ] Sandbox/limites testados quando o isolamento ou execução do engine é alterado
[ ] Retry/DLQ/shutdown testados quando a fila ou ciclo de vida do processo é alterado
[ ] Smoke test executado quando o fluxo crítico é alterado
[ ] RTM atualizada quando requisito ou critério de aceitação é alterado
[ ] ADR criado/atualizado quando uma decisão arquitetural é introduzida ou alterada
```

## 40.1. Documentação mínima obrigatória

"Documentação atualizada" acima era genérico demais pra um projeto solo — sem um mínimo explícito, é fácil pular.

**README.md deve conter, sempre:**

- pré-requisitos com as versões exatas efetivamente distribuídas/suportadas pelo release; o README não pode usar versões fictícias, aproximadas ou “latest”;
- declarar que o projeto seleciona a versão estável mais recente compatível das dependências diretas no momento do release, conforme a política da seção 6.6, sem exigir atualização manual de dependências transitivas;
- passo a passo de instalação (idealmente reduzido a um único comando via `justfile`, ver Apêndice C);
- aviso de performance: hardware de referência é baixa especificação (seção 36.3); em máquina mais potente, ajustar `CARGO_BUILD_JOBS` para o número de núcleos disponíveis.

**Comentário de código:**

- funções públicas do engine (Rust) documentadas com `///`, incluindo por que uma função é `async` ou usa `spawn_blocking` quando isso não for óbvio pelo nome;
- módulos e funções relevantes da aplicação Rust documentados com `///` quando a intenção, invariantes ou motivo não forem óbvios pelo código.

Comentário que só repete a assinatura da função ("retorna um resultado") não conta — deve explicar decisão ou motivo, na mesma linha de raciocínio das marcações DECISÃO/HIPÓTESE já usadas neste documento.

---

## 40.2. Governança de mudanças

### ADRs

Decisões arquiteturais que alterem estrutura, segurança, persistência, protocolo do engine ou modelo de deployment devem possuir um ADR em `docs/adrs/`. Cada ADR deve conter:

- contexto;
- alternativas consideradas;
- decisão;
- consequências;
- data/versão;
- vínculo com requisitos/CRITÉRIOS afetados.

ADRs são documentação de decisão; não substituem a especificação principal.

### Versionamento

**DECISÃO:** releases do produto seguem SemVer `MAJOR.MINOR.PATCH`, sem impedir que schema, engine e ruleset possuam seus próprios identificadores dentro do manifesto.

- MAJOR: mudança incompatível com contrato público ou procedimento de operação;
- MINOR: funcionalidade compatível;
- PATCH: correção compatível.

A versão do documento de especificação e a versão do software são relacionadas, mas não precisam ser numericamente idênticas. A interpretação de SemVer segue o contrato publicado pelo projeto. [23]

### CHANGELOG

`CHANGELOG.md` segue formato compatível com Keep a Changelog e registra mudanças relevantes, migrações, incompatibilidades e requisitos de atualização. [20]

### Matriz de rastreabilidade

A RTM deve relacionar:

```text
Demanda → RF → Critério de aceitação → Caso de uso → Teste → Evidência do teste
```

A matriz é derivada da seção 39 e deve ser atualizada quando requisitos ou critérios mudarem.

### Contribuição e hooks

`CONTRIBUTING.md` é opcional enquanto o projeto for fechado ao grupo acadêmico. Caso o repositório aceite contribuições externas, passa a ser obrigatório e deve definir código, testes, segurança e processo de revisão.

Hooks locais de pre-commit podem ser usados como conveniência, mas **não são a autoridade de qualidade**. Verificações críticas devem continuar disponíveis por comandos explícitos (`just check`, `just test`, `just audit`, `just perf`) para funcionarem em ambientes sem hooks instalados.

# 41. Licenciamento e SBOM
<!-- @section id="41" slug="licenciamento-e-sbom" -->

Esta seção é **governança técnica**, não parecer jurídico.

## 41.1. O que deve ser gerado

- `LICENSE` do código próprio;
- `LICENSES_THIRD_PARTY.md`;
- `SBOM.json` em formato CycloneDX ou SPDX, com versão do formato fixada por release;
- versões exatas;
- hashes dos binários distribuídos;
- hashes/digests das imagens e lockfiles;
- origem dos componentes;
- avisos de copyright;
- procedimentos para atualização;
- declaração das versões exatas utilizadas no release e evidência de que as dependências diretas correspondem às versões estáveis mais recentes compatíveis adotadas pelo projeto conforme a seção 6.6; dependências transitivas devem ser reportadas conforme a resolução efetiva do `Cargo.lock`.

## 41.2. Componentes com atenção especial

### Axum / Askama / SQLx

As versões efetivamente distribuídas dessas crates devem ser registradas no `Cargo.lock` e no SBOM. Para dependências diretas, devem corresponder à versão estável mais recente compatível adotada para o release, salvo exceção documentada conforme a seção 6.6.1. Dependências transitivas seguem a resolução efetiva do `Cargo.lock`. As licenças devem ser conferidas por versão efetivamente empacotada no release; esta seção não constitui parecer jurídico. [1][15][16]

### `serde` / `serde_json`

As versões efetivamente distribuídas devem ser registradas no `Cargo.lock` e no SBOM. Para dependências diretas, aplica-se a política de versão estável mais recente compatível; dependências transitivas seguem a resolução efetiva do `Cargo.lock`. Não fixar neste documento uma licença ou composição sem verificar a versão efetivamente empacotada no release.

### YARA-Forge e regras agregadas

Ver seção 9.1: a licença GPL-3.0 do repositório YARA-Forge não se estende automaticamente a cada regra agregada. Para o SBOM, isso significa registrar proveniência, licença e termos de origem por regra incorporada, não apenas a licença do agregador. [17]

### Velociraptor

- Fase 4;
- não modificar no projeto;
- conservar LICENSE/NOTICE;
- registrar versão e checksum;
- revisar obrigações da licença conforme forma efetiva de distribuição.

### ClamAV e rulesets

Ver seção 12 para o processo completo de habilitação. Para o SBOM:

- ClamAV é opcional;
- rulesets de terceiros não devem ser incorporados sem inventário de licença/termos;
- não assumir que “gratuito” significa “licença permissiva”.

### Podman

Podman é apenas o runtime de containers e não deve ser confundido com dependência do código de negócio.

## 41.3. Validação jurídica antes de distribuição

**DECISÃO:** o inventário técnico desta seção (SBOM, licenças por componente, proveniência de regras) é pré-requisito para a validação jurídica, não um substituto dela. Nenhuma distribuição fora do ambiente de desenvolvimento/demonstração controlada deve ocorrer antes de uma pessoa com autoridade jurídica na Avivatec/SENAI revisar formalmente este inventário — em especial os componentes GPL-3.0 (YARA-Forge, seção 9.1) frente ao modelo efetivo de distribuição escolhido.

Esta seção não determina quem é essa pessoa nem quando isso ocorre; isso é decisão de negócio externa ao escopo deste documento (seção 48). O que este documento decide é que a etapa existe como gate obrigatório antes de distribuição, não como formalidade opcional.

### 41.4. Rejeição de Hayabusa e Chainsaw para o produto

**DECISÃO:** Hayabusa e Chainsaw não serão incorporados ao produto como dependências, binários distribuídos, submódulos, regras, fixtures operacionais, componentes de container ou ferramentas de runtime.

A decisão é uma medida conservadora de engenharia e supply-chain para preservar a possibilidade de distribuição comercial/licenciamento próprio e manter controle sobre os componentes efetivamente distribuídos. Não constitui parecer jurídico nem afirma que todo uso externo dessas ferramentas seria juridicamente impossível.

O software Hayabusa declara AGPLv3 e seu repositório de regras declara Detection Rule License (DRL) 1.1; o repositório oficial de Chainsaw declara GPL-3.0. Essas informações servem como evidência técnica do licenciamento dos artefatos consultados e devem ser revalidadas antes de qualquer futura consideração.

A rejeição também evita incorporar uma suíte externa completa onde o projeto já possui arquitetura própria para EVTX, regras e análise. A reintrodução de qualquer uma dessas ferramentas só pode ocorrer por nova ADR, necessidade demonstrada, inventário técnico de licença/proveniência e validação jurídica do modelo efetivo de distribuição.

---

# 42. Dataset de demonstração
<!-- @section id="42" slug="dataset-de-demonstracao" -->

Características:

- sintético ou devidamente autorizado;
- sanitizado;
- reproduzível;
- checksum;
- sem segredo real;
- sem PII real desnecessária.

Cenário sugerido:

```text
Phishing/BEC
    ↓
Credencial comprometida
    ↓
Login anômalo
    ↓
Execução pós-comprometimento
    ↓
Escalonamento / lateral movement
    ↓
Coleta / tentativa de exfiltração
    ↓
Finding
    ↓
ATT&CK
    ↓
NIST Gap
    ↓
Remediation
    ↓
Relatórios
```

Nenhuma etapa deve depender de comportamento aleatório durante a apresentação.

---

# 43. Apresentação de 5 minutos
<!-- @section id="43" slug="apresentacao-de-5-minutos" -->

## 43.1. Abertura — 30 s

Problema → demanda → objetivo.

## 43.2. DFIR — 90 s

Upload → SHA-256 → cadeia → EVTX → timeline → YARA-X → IOC → Finding.

## 43.3. NIST — 60 s

Current → Gap → Target → Prioridade.

## 43.4. Entregáveis — 60 s

DFIR → vulnerabilidades/ATT&CK → NIST → hardening → criptografia.

## 43.5. Encerramento — 30 s

Local → rastreável → offline → demonstrável.

---

# 44. Plano B
<!-- @section id="44" slug="plano-b" -->

```text
localhost
   ↓
SQLite
   ↓
dataset BEC
   ↓
regras locais
   ↓
engine Rust
   ↓
Typst
   ↓
PDF
```

Não depender de Internet, API externa, CDN, cloud ou credencial de terceiro.

---

# 45. Diretrizes obrigatórias para a LLM implementadora
<!-- @section id="45" slug="diretrizes-obrigatorias-para-a-llm-implementadora" -->

```xml
<regras-para-llm>
  <regra id="LLM-001">
    Antes de implementar, identifique requisito, caso de uso, dependências e critério de aceitação.
  </regra>

  <regra id="LLM-002">
    Não invente requisitos de negócio.
  </regra>

  <regra id="LLM-003">
    Decisões de engenharia podem ser substituídas somente com justificativa explícita.
  </regra>

  <regra id="LLM-004">
    Nunca modificar a evidência forense original.
  </regra>

  <regra id="LLM-005">
    Nunca descartar silenciosamente erro de parsing.
  </regra>

  <regra id="LLM-006">
    Não interpretar ausência de detecção como ausência de ameaça.
  </regra>

  <regra id="LLM-007">
    Não tratar NIST Tiers como nota automática 1–4 por Subcategory.
  </regra>

  <regra id="LLM-008">
    Não criar mapping MITRE ATT&amp;CK sem evidência/justificativa.
  </regra>

  <regra id="LLM-009">
    Não executar comandos destrutivos ou hardening em produção como efeito colateral.
  </regra>

  <regra id="LLM-010">
    O motor Rust não deve receber segredos do domínio nem acesso direto ao banco.
  </regra>

  <regra id="LLM-011">
    O engine deve trabalhar com limites explícitos de memória, tempo e tamanho de entrada.
  </regra>

  <regra id="LLM-012">
    Não afirmar que benchmark externo é benchmark do projeto.
  </regra>

  <regra id="LLM-013">
    Não introduzir componente de terceiros sem registrar versão, origem e licença.
  </regra>

  <regra id="LLM-014">
    Não incorporar secret, token, chave privada ou PII real ao repositório ou dataset.
  </regra>

  <regra id="LLM-015">
    Preferir soluções simples e determinísticas no MVP.
  </regra>

  <regra id="LLM-016">
    Não transformar regra de licença em parecer jurídico.
  </regra>

  <regra id="LLM-017">
    Uma funcionalidade só pode ser descrita como concluída quando os critérios de aceitação e testes relevantes estiverem atendidos.
  </regra>

  <regra id="LLM-018">
    Toda mudança de arquitetura deve atualizar o histórico de decisões e os critérios afetados.
  </regra>

  <regra id="LLM-019">
    Nunca atualizar aplicação, schema, ruleset ou bundle em ambiente de execução sem seguir o ciclo de atualização controlada da seção 29.4.
  </regra>

  <regra id="LLM-020">
    O processo Rust deve permanecer supervisionado pelo Worker e nunca receber acesso direto ao SQLite, secrets do aplicativo ou caminhos arbitrários fornecidos pelo usuário.
  </regra>
  <regra id="LLM-021">
    Não tratar licença agregadora de YARA-Forge como licença automática de cada regra importada; preservar a proveniência e os termos da fonte efetiva.
  </regra>
  <regra id="LLM-022">
    Não classificar rulesets de terceiros do ClamAV como parte do ruleset oficial do projeto.
  </regra>
  <regra id="LLM-023">
    Não inventar percentual de progresso quando a fonte não fornece progresso mensurável.
  </regra>
  <regra id="LLM-024">
    Não executar manutenção pesada de SQLite por calendário fixo sem necessidade técnica demonstrada.
  </regra>
  <regra id="LLM-025">
    Não converter um job PROCESSING em COMPLETED após shutdown/restart sem validação explícita do resultado.
  </regra>
  <regra id="LLM-026">
    Priorize clareza, correção, segurança e rastreabilidade antes de concisão; não introduza complexidade ou abstrações sem necessidade demonstrada.
  </regra>
  <regra id="LLM-027">
    Antes de alterar código existente, preserve o comportamento não relacionado à tarefa e evite refatorações oportunistas.
  </regra>
  <regra id="LLM-028">
    Comentários devem explicar principalmente decisões, invariantes e razões não óbvias; não duplicar o que o código já expressa claramente.
  </regra>
  <regra id="LLM-029">
    Operações sensíveis devem permanecer rastreáveis desde entrada, validação e autorização até efeito e auditoria.
  </regra>
  <regra id="LLM-030">
    Respeite as responsabilidades arquiteturais da seção 37.2; não mova regras de negócio para infraestrutura nem detalhe técnico para o domínio apenas por conveniência local.
  </regra>
  <regra id="LLM-031">
    A árvore de arquivos é uma referência flexível, mas novas camadas, módulos compartilhados ou mudanças estruturais exigem necessidade demonstrada e atualização dos artefatos afetados.
  </regra>
</regras-para-llm>
```

---

# 46. Protocolo de implementação
<!-- @section id="46" slug="protocolo-de-implementacao" -->

1. Ler a especificação inteira.
2. Identificar requisitos e decisões.
3. Detectar conflitos.
4. Registrar pendências.
5. Modelar domínio.
6. Modelar banco.
7. Criar Ports/Interfaces.
8. Implementar Web/App.
9. Implementar Engine Rust.
10. Implementar protocolo JSONL.
11. Implementar testes.
12. Integrar Podman.
13. Validar segurança.
14. Executar benchmarks.
15. Validar atualização/rollback quando a etapa alterar release, schema ou bundle.
16. Atualizar documentação.
17. Informar exatamente o que foi concluído.

---

# 47. Protocolo de resposta da LLM durante desenvolvimento
<!-- @section id="47" slug="protocolo-de-resposta-da-llm-durante-desenvolvimento" -->

A resposta de uma LLM implementadora deve distinguir claramente **o que existe, o que está integrado e o que foi verificado**. O formato abaixo é mínimo; campos podem ser expandidos, mas não devem ser removidos quando forem aplicáveis.

```xml
<implementation-report version="1.0">
  <status value="concluido|em-andamento|bloqueado"/>
  <changes>
    <file path="..." action="created|modified|removed"/>
  </changes>
  <traceability>
    <requirement ref="RF-xxx" status="implemented|partial|not-verified"/>
    <decision ref="Dxx" status="preserved|changed"/>
    <acceptance-criterion ref="CA-xxx" status="met|not-met|not-tested"/>
  </traceability>
  <tests>
    <test id="..." type="unit|integration|security|runtime|property" status="passed|failed|not-run"/>
  </tests>
  <gates>
    <gate id="GATE-*" status="PASS|FAIL|BLOCKED|PENDING|WAIVED|STALE" evidence="EVD-*"/>
  </gates>
  <evidence>
    <record id="EVD-*" gate="GATE-*" commit_tree="..." toolchain="..." executed_at="..." result="..." artifact="..."/>
  </evidence>
  <waivers>
    <waiver id="WVR-*" gate="GATE-*" status="approved" reason="..." approved_by="..." approved_at="..." valid_until="..."/>
  </waivers>
  <benchmarks>
    <benchmark id="..." status="executed|not-run" dataset_sha256="..." environment="..." result="..."/>
  </benchmarks>
  <pending>
    <item id="..." reason="..."/>
  </pending>
  <decisions>
    <decision ref="Dxx" reason="..."/>
  </decisions>
  <documentation-updated>
    <file path="..."/>
  </documentation-updated>
</implementation-report>
```

**Regra:** `passed` significa que a verificação correspondente foi realmente executada e produziu evidência. `implemented` significa que o caminho de código existe; não implica build/runtime verificado. `not-verified`/`not-run` devem permanecer explícitos quando um gate estiver indisponível.

---

# 48. Pendências externas reais
<!-- @section id="48" slug="pendencias-externas-reais" -->

Estas não são pendências do “documento original”, que já foi fornecido. São informações que podem ainda depender da Avivatec/SENAI:

- infraestrutura real autorizada para testes;
- dados reais disponibilizados;
- requisitos adicionais de governança da empresa;
- critérios/pesos internos da banca não presentes na demanda fornecida;
- ambiente operacional de produção, caso exista intenção de uso além da demonstração;
- exigências jurídicas específicas de distribuição comercial.

---

# 49. Metadados da demanda
<!-- @section id="49" slug="metadados-da-demanda" -->

```xml
<demanda-original>
  <origem>SAGA SENAI de Inovação</origem>
  <tipo>Projeto Integrador 2026/01</tipo>
  <escola>(SP) 1.32 Escola SENAI de Informática</escola>
  <empresa>Avivatec Soluções em Tecnologia da Informação Ltda.</empresa>
  <codigo-demanda>12412</codigo-demanda>
  <area>Tecnologia da Informação</area>
  <estado>SP</estado>
  <data-cadastro>04/03/2026 21:26</data-cadastro>
  <inicio-vigencia>04/03/2026</inicio-vigencia>
  <vigencia>04/03/2027</vigencia>
</demanda-original>
```

---

# 50. Decisões finais consolidadas
<!-- @section id="50" slug="decisoes-finais-consolidadas" -->

```xml
<decisoes-arquiteturais>
  <arquitetura>Monolito modular + motor de análise Rust em processo separado</arquitetura>
  <web>Rust + Axum + Tokio + HTTP/JSON</web>
  <frontend>TypeScript compilado para browser; sem runtime Node/Bun/Deno em produção</frontend>
  <frontend-interactivity>HTML server-rendered para fluxos simples; TypeScript para estado/visualização local; HTTP/JSON para dados e SSE somente quando necessário</frontend-interactivity>
  <persistencia>SQLite + WAL no contexto de worker único</persistencia>
  <engine>Rust</engine>
  <parsing>EVTX incremental</parsing>
  <detection>YARA-X</detection>
  <data-processing>Polars via API Rust</data-processing>
  <engine-protocol>JSONL sobre stdin/stdout via subprocesso em produção e testes</engine-protocol>
  <ipc>stdin/stdout do subprocesso; sem socket ou API de rede no engine do MVP</ipc>
  <ruleset-mvp>Ruleset próprio e curado; feeds públicos apenas como fonte opcional de curadoria</ruleset-mvp>
  <database-policy>SQLite + WAL + FK + FULL sync + busy timeout</database-policy>
  <crypto-at-rest>Proteção de volume/filesystem como mecanismo primário no MVP; field encryption apenas quando justificada</crypto-at-rest>
  <signing>Ed25519 destacada quando assinatura estiver habilitada; chave separada por finalidade</signing>
  <runtime-container>Podman rootless; runtime OCI não fixado</runtime-container>
  <linux-service>Quadlet/systemd</linux-service>
  <reporting>Typst + exportação Markdown/JSON</reporting>
  <remote-collection>Velociraptor, Fase 4</remote-collection>
  <hardening>Geração de Ansible, sem aplicação automática em produção</hardening>
  <nist>Current Profile + Target Profile + Gap Analysis + Tiers contextualizados</nist>
  <forensics>Original preservada + chain of custody verificável</forensics>
  <offline>Suportado no MVP</offline>
  <microservices>Não utilizados no MVP</microservices>
  <deployment>Um único container rootless contendo app e processo engine separado</deployment>
  <engine-isolation>Processo separado + sandbox bubblewrap por job dentro do container; defesa em profundidade, não VM nem fronteira absoluta</engine-isolation>
  <updates>Atualização controlada; manifesto, checksum, assinatura quando habilitada, preflight, backup, migration, health check, smoke test e rollback</updates>
  <auto-update>Desabilitado no MVP</auto-update>
  <external-dfir-tools>Hayabusa e Chainsaw rejeitados como componentes, dependências, regras ou binários distribuídos do produto; reavaliação exige ADR e validação jurídica aplicável</external-dfir-tools>
</decisoes-arquiteturais>
```

## 50.1. Sugestões avaliadas e não incorporadas

| Sugestão | Decisão | Justificativa resumida |
|---|---|---|
| Periodicidade trimestral obrigatória do ruleset | REJEITADO | Atualizar por evento relevante ou ciclo realmente sustentável; calendário arbitrário não é requisito. |
| Sanesecurity como base obrigatória/BSD | REJEITADO | Termos da fonte não sustentam a simplificação; usar somente com origem e termos efetivamente registrados. [19] |
| `VACUUM`/`REINDEX` mensal | REJEITADO | Manutenção SQLite deve ser acionada por evidência, não por calendário. |
| D3.js/Cytoscape obrigatório | REJEITADO | Grafo é requisito; biblioteca específica depende de necessidade/benchmark. MVP usa SVG controlado pelo servidor. |
| Dataset real obrigatório | REJEITADO | Confidencialidade/privacidade; demonstração usa dataset sintético/autorizado. |
| Rate limiting somente em exposição pública | REJEITADO | Autenticação continua sendo fronteira sensível mesmo em localhost; apenas o perfil pode ser ajustado. |
| `podman-compose.yml` | REJEITADO | `compose.yaml` mantém neutralidade de ferramenta; Podman rootless é runtime de referência. |
| Hayabusa/Chainsaw como motores externos do MVP | REJEITADO | Preservar possibilidade de distribuição comercial/licenciamento próprio; AGPLv3/DRL 1.1 (Hayabusa) e GPL-3.0 (Chainsaw) aumentam a necessidade de análise jurídica e supply-chain; o projeto já possui arquitetura própria e o princípio de necessidade demonstrada impede adicionar suítes completas sem requisito concreto. |
| Eliminar processo separado do engine | REJEITADO | Separação preserva contenção de crash, timeout/cancelamento e sandbox por job. |
| Tour JavaScript obrigatório | REJEITADO | Guia do Usuário + onboarding leve atendem o MVP; tour depende de necessidade demonstrada. |
| Rust/WASM no frontend | REJEITADO | Não reduz complexidade e não há benchmark que demonstre necessidade; TypeScript continua suficiente para interações locais. |

As justificativas técnicas completas permanecem nas seções que definem as respectivas decisões (stack, UX, sandbox, ruleset, performance, licenciamento e offline). D04/D26 permanecem como decisões compatíveis com a nova fronteira; D40/D41/D42 complementam e refinam o baseline.

---


# 51. Estado do documento
<!-- @section id="51" slug="estado-do-documento" -->

```xml
<estado-do-documento>
  <versao>6.4.1</versao>
  <maturidade>ESPECIFICAÇÃO DE ARQUITETURA FECHADA E BASE CONTROLADA DE IMPLEMENTAÇÃO</maturidade>
  <implementacao>Não implica código já implementado</implementacao>
  <demanda-original>Incorporada</demanda-original>
  <arquitetura>Backend confiável em Rust; frontend server-rendered com TypeScript compilado; tooling Python/Shell fora do runtime; engine separado e sandboxado por job</arquitetura>
  <benchmarks>Devem ser realizados na implementação</benchmarks>
  <analise-de-licencas>Inventário técnico; validação jurídica final pendente do modelo efetivo de distribuição</analise-de-licencas>
  <status>APTA PARA IMPLEMENTAÇÃO CONTROLADA — ARQUITETURA FECHADA</status>
</estado-do-documento>
```

---

# 52. Princípio final
<!-- @section id="52" slug="principio-final" -->

A solução deve ser:

```text
rastreável
+
testável
+
segura
+
determinística
+
auditável
+
fiel à demanda
+
preservadora da evidência
+
executável localmente
+
offline-ready
+
compreensível por humanos
```

Performance é uma **propriedade a ser medida**, não uma promessa colocada no texto.

Conformidade legal é uma **propriedade a ser validada no modelo efetivo de distribuição**, não uma declaração automática do documento.

O projeto deve privilegiar **correção, rastreabilidade e verificabilidade sobre quantidade de funcionalidades**.

---

## 52.1. Refinamentos aplicados na revisão de implementação — 2026-08-17
<!-- @section id="52-1" slug="refinamentos-revisao-2026-08-17" -->

A revisão do artefato de implementação da Fase 0 identificou ajustes de segurança, coerência de release e verificabilidade. Estes refinamentos não alteram a arquitetura consolidada; apenas tornam mais explícita e verificável a implementação já decidida.

### R-001 — Coerência entre manifesto, lockfile e SBOM

O conjunto de release deve ser tratado como uma unidade coerente:

```text
Cargo.toml
    ↓
Cargo.lock
    ↓
cargo metadata --locked / cargo tree --locked
    ↓
SBOM gerado pelo workspace efetivamente resolvido
```

O `Cargo.lock` permanece a autoridade sobre as versões resolvidas. O `SBOM.json` não deve ser corrigido manualmente para mascarar divergências; deve ser regenerado após a resolução final do Cargo. O validador da Fase 0 deve detectar divergências entre manifesto, lockfile e SBOM quando o ambiente possuir Cargo.

### R-002 — Autenticação e contenção de SQLite

A verificação Argon2 não deve ocorrer enquanto a transação `BEGIN IMMEDIATE` estiver bloqueando outros escritores. O fluxo recomendado é:

```text
consulta da credencial
        ↓
verificação Argon2 fora da transação de escrita
        ↓
BEGIN IMMEDIATE
        ↓
releitura/validação do estado mutável
        ↓
UPDATE de sucesso/falha/bloqueio
        ↓
COMMIT
```

Para reduzir enumeração temporal, usuário inexistente deve realizar verificação contra um hash Argon2 dummy controlado. A senha recebida deve possuir limite máximo de bytes antes de qualquer operação de hashing/verificação.

### R-003 — Verificação da cadeia de auditoria em lotes

A verificação da auditoria deve processar registros incrementalmente em lotes limitados, em vez de carregar o histórico completo de uma vez. O resultado deve continuar comparando contagem, sequência, `previous_hash`, hash recalculado e `head_hash` persistido.

### R-004 — Sandbox por job com mount mínimo

O `bubblewrap` deve montar somente o executável efetivamente usado pelo engine, evitando expor desnecessariamente todo o diretório pai. O isolamento continua sendo defesa em profundidade sobre o container rootless e não deve ser descrito como equivalente a VM ou segundo host.

### R-005 — Endurecimento do Compose de desenvolvimento

O `compose.yaml` de desenvolvimento deve manter propriedades próximas às do ambiente de referência sempre que compatível com a ferramenta: usuário não-root, `read_only`, `cap_drop: ALL` e `no-new-privileges`. Isso não substitui os gates reais do Quadlet/Podman de produção.

### R-006 — Testes adversariais obrigatórios para o release

A suíte de Fase 0 deve possuir casos para entradas JSONL truncadas/malformadas, limites excessivos, versões incompatíveis, mensagens fora de ordem, EOF prematuro, timeout, processo persistente, stderr excessivo e tentativas de acesso fora dos mounts permitidos pelo sandbox.

### R-007 — Linguagem de status do storage

A implementação de infraestrutura de storage e referências lógicas não deve ser apresentada como equivalente à preservação forense completa de evidências. O status deve distinguir claramente infraestrutura de armazenamento, ingestão/preservação, análise semântica e cadeia de custódia implementada.

### R-008 — Estado de release

A versão do documento, `Cargo.lock`, SBOM, changelog, RTM, status e ADRs devem identificar o mesmo estado lógico de release. A versão declarada no documento é **6.4.1**. Qualquer divergência deve bloquear o fechamento do release.

### R-009 — Evidência não é identificada pelo lockfile

`Cargo.lock` não prova qual versão de Cargo o gerou. Releases e gates de compilação devem registrar o toolchain efetivamente executado, incluindo `rustc --version --verbose` e `cargo --version --verbose`; `rust-toolchain.toml` declara a referência, não substitui a evidência.

### R-010 — Estado e exceção são diferentes

`PASS` representa verificação executada; `WAIVED` representa exceção de governança; `PENDING`, `BLOCKED` e `STALE` não satisfazem a transição. Qualquer evidência stale precisa ser reexecutada depois da mudança que a invalidou.


# 53. Histórico de revisões

| Versão | Origem/autor | Síntese |
|---|---|---|
| 6.4.1 | Hardening preventivo e congelamento pré-testes — 2026-08-17 | Bloqueio de autenticação por backoff passa a ser aplicado efetivamente; JSON da timeline é escapado para contexto HTML; páginas autenticadas usam `Cache-Control: no-store`; verificação do frontend compara artefato versionado com transpile independente; `DFIR_HTTPS=false` fica restrito a binds loopback por padrão, com exceção explícita apenas para tooling de desenvolvimento/teste; Deno de referência atualizado para 2.9.4. |
| 6.4.0 | Refinamento arquitetural de runtime e tooling — 2026-08-17 | Backend confiável permanece Rust; TypeScript torna-se frontend compilado; Deno é toolchain de build/dev; CPython/Shell são tooling de engenharia/QA; HTTP/JSON + SSE substituem a necessidade de WebSocket no MVP; renderer de timeline local evita dependência externa obrigatória na Fase 0. |
| 6.3.13-r3 | Auditoria preventiva pré-testes — 2026-08-17 | Normalização final do RTM, evidências, toolchain real e staleness; Definition of Ready endurecido. |
| 6.3.13-r2 | Revisão preventiva — 2026-08-17 | PASS/WAIVED/BLOCKED/PENDING/STALE, evidência identificada, IDs imutáveis e política de CI sem alegação de workflow inexistente. |
| 6.3.13-r1 | Revisão de implementação — 2026-08-17 | Coerência Cargo/lock/SBOM, autenticação fora de transação de hash, limites de senha, hash dummy, auditoria incremental, sandbox mínimo e Compose endurecido. |
| 6.3.13 | Congelamento técnico — 2026-08-17 | Lockfile/grafo como autoridade operacional; gates de supply chain e revisão das famílias RNG. |
| 6.3.12 | Refino RNG — 2026-08-17 | `getrandom 0.4.3` como RNG direto; `rand`/`rand_core` fora da fronteira de autenticação. |
| 6.3.11 | Build/reprodutibilidade — 2026-08-17 | `rand`/`rand_core` removidos como dependências diretas; Argon2 sem feature `rand`; migration centralizada. |
| 6.3.10 | Build/reprodutibilidade — 2026-08-17 | Iteração anterior da correção RNG/Argon2. |
| 6.3.8 | Refino de engenharia — 2026-08-17 | MiniJinja, Tower/Tower-HTTP, gates de supply chain, AppConfig e erros estruturados. |
| 6.3.3 | Revisão de distribuição — 2026-08-17 | Rejeição formal de Hayabusa/Chainsaw como componentes do produto. |
| 6.3.0 | Compressão estrutural | Redução de redundância sem remover conteúdo normativo. |
| 6.2.0 | Dependências/build | Distinção direta/transitiva, MSRV/toolchain e estratégia offline do SQLx. |
| 6.0.0 | Fechamento técnico | `tracing`, `zeroize`, CI gates, proveniência, segurança e governança de distribuição. |
| 5.5.0 | Consistência pós-migração | Limpeza de resíduos JS e correções de sandbox/referências. |
| 5.4.0 | Arquitetura | Backend migrado de Bun/TypeScript/Hono/Drizzle/Zod para Rust/Axum/Tokio/Askama/SQLx. |
| 5.3.0 | Escopo de parsing | EVTX como parser automático do MVP; parsers futuros em catálogo. |
| 5.2.0 | UX/frontend | Interações leves no frontend sem SPA. |
| 5.0.0 | Consolidação | Curadoria YARA-X, ClamAV, UX, benchmark, ADRs, SemVer, CHANGELOG e RTM. |
| 4.3.0 | Revisão operacional | Contrato JSONL, referências lógicas, limites, idempotência, DLQ/recovery e auditoria. |
| 4.2.0 | Revisão profunda | Correções contra demanda, DFIR, parsing, backup/migrations e segurança. |
| 4.1.0 | Revisão meticulosa | Índice normativo e sandbox `bubblewrap` por job. |
| 3.3.0 | Revisão final | Monólito modular, `.dfirpkg`, assinatura, archive-bomb, bootstrap admin e entregáveis. |
| 3.0–3.2 | Revisões anteriores | Rust engine, IOCs, Sigma, licenças, Velociraptor e SBOM. |
| 2.2.0 | Revisão anterior | Podman, Typst, BEC, PCAP, YARA-X e SQLite/WAL. |
| 2.1.0 | Revisão anterior | Demanda original, rastreabilidade e regras para LLM. |
| 2.0.0 | Revisão anterior | Estrutura para implementação orientada por LLM. |
| 1.0.0 | Documento inicial | Especificação inicial. |

**Regra:** o histórico resume mudanças; requisitos, decisões, critérios e procedimentos completos permanecem nas seções normativas correspondentes. Em conflito, o corpo normativo prevalece.

## Referências técnicas adicionais verificadas nesta revisão

38. Deno — estabilidade, releases e LTS 2.9: https://docs.deno.com/runtime/fundamentals/stability_and_releases/
39. Deno — `deno transpile`: https://docs.deno.com/runtime/reference/cli/transpile/
40. Python 3.14.6 — documentação/changelog oficial: https://docs.python.org/3.14/whatsnew/changelog.html

# Apêndice A. Guia do Usuário — estrutura mínima
<!-- @section id="apx-a" slug="guia-do-usuario-estrutura-minima" -->

Os 5 entregáveis da seção 4 documentam **um caso**. Faltava um documento sobre **a ferramenta em si** — como instalar, operar e recuperar o sistema. É conteúdo diferente, não deve ser misturado com os entregáveis forenses.

**DECISÃO:** existe um Guia do Usuário, versionado junto com o software (não por incidente), gerado pelo mesmo pipeline Typst dos relatórios (seção 30) — sem inventar uma segunda ferramenta de documentação.

Estrutura mínima obrigatória:

1. **Instalação e pré-requisitos** — espelha o README (seção 40.1), em linguagem para operador, não para desenvolvedor;
2. **Bootstrap do primeiro administrador** — procedimento já exigido em outra parte do documento; o guia só precisa explicá-lo passo a passo com capturas/exemplos;
3. **Papéis e permissões** — o que cada papel RBAC (seção 23) pode e não pode fazer, em linguagem operacional;
4. **Ciclo de vida de um caso** — criar incidente → subir evidência → acompanhar pipeline/fila → interpretar resultado → gerar relatório. Esse é o fluxo que um analista realmente segue no dia a dia e é a seção que mais precisa de exemplo concreto, não só descrição;
5. **Backup e restauração** — procedimento operacional da seção 27.2, sem o detalhe técnico interno, focado em "como eu, operador, executo isso";
6. **Modo Air-Gapped** — como operar sem internet (seção 29), incluindo atualização manual de rulesets e assinaturas habilitadas pelo administrador;
7. **Atualização e rollback** — como validar um bundle, criar backup, aplicar a atualização, confirmar health/smoke test e executar rollback quando necessário;
8. **Troubleshooting comum** — os erros mais prováveis de aparecer (engine indisponível, fila travada, disco cheio, atualização incompleta — ligar ao health check da seção 34.1 e ao ciclo de atualização da seção 29.4) e o que fazer em cada um;
9. **FAQ** — preenchido conforme dúvidas reais aparecerem na prática; não deve ser inventado especulativamente agora.

---

# Apêndice B. Compatibilidade multiplataforma — Windows e WSL2
<!-- @section id="apx-b" slug="compatibilidade-multiplataforma-windows-e-wsl2" -->

O documento nunca esclareceu isso, e existem três perguntas diferentes escondidas numa só:

## B.1. Onde o servidor roda

**DECISÃO:** Linux nativo é o alvo de implantação suportado. Toda a base de containerização (seção 28) depende de Podman rootless + Quadlet/systemd — nenhum dos dois é nativo no Windows.

Windows via **WSL2 é caminho best-effort, não oficialmente suportado no MVP**, com ressalvas conhecidas:

- Podman rootless dentro do WSL2 tem histórico de fricção com namespaces de usuário aninhados (namespace do WSL2 dentro do namespace do rootless Podman);
- Quadlet depende de `systemd --user`, que **não vem habilitado por padrão** no WSL2 — exige `systemd=true` explícito em `/etc/wsl.conf` e reinício da distro;
- se o objetivo for só desenvolvimento local (não implantação real), WSL2 com essas duas ressalvas resolvidas é aceitável; para operação real de um caso, Linux nativo é a recomendação.

`scripts/setup.ps1` (seção 37) existe para esse cenário específico: preparar o WSL2 do lado Windows (habilitar a feature, garantir `systemd=true`, instalar a distro) — não para rodar o app nativamente no Windows.

## B.2. De onde vem a evidência

Windows é **primeira classe como origem de evidência**, independente de onde o servidor roda — é justamente o formato que o parser EVTX (seção 10) foi desenhado para consumir. Isso é completamente independente da pergunta B.1.

## B.3. Quem acessa o sistema

Irrelevante. A interface é server-rendered, com TypeScript apenas para interações locais — qualquer sistema operacional com navegador funciona como cliente, sem instalação. Um analista em Windows, macOS ou Linux acessa o mesmo jeito.

---

# Apêndice C. Configuração do ambiente de desenvolvimento
<!-- @section id="apx-c" slug="configuracao-do-ambiente-de-desenvolvimento" -->

Consolidação do ambiente de desenvolvimento Rust do projeto, sobre o fluxo de trabalho padrão do Cargo. [9] O objetivo é manter o fluxo reproduzível e simples mesmo em hardware de baixa especificação.

## C.1. `justfile`

**RECOMENDADO:** centraliza os comandos do projeto; não é dependência do produto.

```just
build:
    cargo build --workspace

check:
    cargo check --workspace --all-targets

run:
    cargo run --bin dfir-app

engine:
    cargo run --bin dfir-engine -- --stdio

test:
    cargo test --workspace

security:
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt --all

bench:
    cargo bench --workspace

smoke:
    just check
    cargo test --workspace smoke -- --nocapture

audit:
    cargo audit

perf:
    cargo bench --workspace
    # cargo bench isoladamente não satisfaz a seção 36.6: o alvo real também deve
    # coletar hardware/OS, dataset+SHA-256, versões e demais metadados antes de
    # publicar o resultado como JSON estruturado.
```

Ferramentas de benchmark externas, como `hyperfine`, podem ser instaladas no ambiente de desenvolvimento, mas não são dependências do produto.

## C.2. `direnv` / `.envrc`

**RECOMENDADO:** opcional. Variáveis devem ser de desenvolvimento e nunca conter segredos reais.

```bash
export CARGO_TARGET_DIR="$PWD/target"
export RUST_BACKTRACE=1
```

## C.3. Rust Analyzer

Configuração de referência deve priorizar `cargo check` e evitar builds completos a cada alteração em hardware de baixa especificação.

## C.4. Linker `mold` (opcional)

Pode acelerar builds incrementais, mas continua sendo opcional e deve ser validado por benchmark antes de virar recomendação do ambiente padrão.

## C.4.1. Frontend e tooling Python

O frontend usa Deno apenas para verificar, formatar, lintar e transpilá-lo para browser. O código TypeScript não usa APIs exclusivas do Deno.

CPython 3.14.6 é a referência para scripts de QA/fixtures/benchmarks/release; scripts não são empacotados na imagem final.

## C.5. Fluxo de build

- `cargo check` como primeira verificação durante desenvolvimento;
- `cargo test --workspace` para alterações de comportamento;
- `cargo clippy --workspace --all-targets -- -D warnings` como gate de qualidade quando aplicável;
- `cargo build --release` para builds de benchmark/empacotamento;
- `cargo fmt --all -- --check` como verificação de formatação.
- `deno check frontend/src/app.ts` e `deno lint frontend/src/app.ts` para o frontend;
- `deno fmt --check frontend/src/app.ts frontend/deno.json`;
- `deno transpile frontend/src/app.ts --outdir frontend/dist`;

## C.6.1. Verificação offline do SQLx

O procedimento exato depende da versão de SQLx adotada no release, mas o resultado obrigatório é fixo: o workspace deve conseguir verificar e compilar suas queries sem Internet. Os metadados de query exigidos pela estratégia escolhida devem estar preparados antes do ambiente Air-Gapped e versionados quando aplicável.

O README e o processo de release devem indicar os comandos concretos usados para preparar, verificar e testar as queries da versão adotada. Não adicionar `sqlx-cli` ou configurar `DATABASE_URL` sem necessidade comprovada.

## C.6. Auditoria de dependências

**DECISÃO:** `cargo audit` é obrigatório antes de release e quando dependências forem adicionadas ou atualizadas. A análise cobre o workspace Rust e deve ser registrada no processo de release. [8]

## C.7. Integração contínua (CI)

**DECISÃO:** quando o repositório possuir integração contínua (CI) com proteção de merge, todo push/PR na branch principal deve executar automaticamente, no mínimo, os gates abaixo antes de permitir merge — os mesmos comandos do `justfile` (C.1), sem duplicar lógica. Até existir um workflow de CI versionado neste repositório, o requisito é uma política-alvo e não deve ser descrito como controle já implementado; os gates locais do `justfile` e o `release-gate` continuam sendo a autoridade operacional disponível.

- `cargo fmt --all -- --check`;
- `just security` (`cargo clippy --workspace --all-targets -- -D warnings`);
- `just test` (`cargo test --workspace`);
- `just audit` (`cargo audit`), obrigatoriamente quando `Cargo.lock` mudar.

`just bench`/`just perf` **não** são gate de PR: variam por hardware do runner e têm custo alto para rodar em todo commit. Rodam sob demanda ou no processo de release (seção 36), nunca bloqueando merge por variação de benchmark.

**RECOMENDADO:** o provedor de CI é ilustrativo, não uma decisão arquitetural travada — qualquer executor que rode os mesmos comandos é equivalente. A existência do texto normativo não implica que um workflow já esteja instalado no repositório.

```yaml
# Ilustrativo — não vincula o projeto a um provedor específico de CI.
on: [push, pull_request]
jobs:
  gates:
    steps:
      - run: cargo fmt --all -- --check
      - run: just security
      - run: just test
      - run: just audit   # obrigatório quando Cargo.lock mudar
```


# Apêndice D. Índice normativo para consumo por LLM
<!-- @section id="apx-d" slug="indice-normativo-para-consumo-por-llm" -->

**DECISÃO:** este índice é derivado do corpo. É navegação auxiliar para LLMs e ferramentas; o corpo do documento permanece a fonte de verdade.

Se o índice divergir do corpo, o corpo prevalece e o índice deve ser regenerado.

```xml
<indice-normativo versao-documento="6.4.1" schema="1.0">
  <secoes faixa-secoes="1-53" appendices="A-F" inclui-subsecoes="true">
    <secao id="1" slug="convencoes-normativas" tipo="section">Convenções normativas</secao>
    <secao id="2" slug="demanda-original-consolidada" tipo="section">Demanda original consolidada</secao>
    <secao id="3" slug="restricoes-da-demanda" tipo="section">Restrições da demanda</secao>
    <secao id="4" slug="entregaveis-e-rastreabilidade" tipo="section">Entregáveis e rastreabilidade</secao>
    <secao id="5" slug="decisoes-arquiteturais-finais" tipo="section">Decisões arquiteturais finais</secao>
    <secao id="6" slug="stack-tecnologica-final" tipo="section">Stack tecnológica final</secao>
    <subsecao id="6.6" secao="6" slug="politica-de-versoes">Política de versões — somente versões estáveis mais recentes</subsecao>
    <subsecao id="6.6.1" secao="6.6" slug="excecao-controlada-por-compatibilidade">Exceção controlada por compatibilidade</subsecao>
    <subsecao id="6.6.2" secao="6.6" slug="dependencias-diretas-transitivas-e-toolchain">Dependências diretas, transitivas e toolchain</subsecao>
    <subsecao id="6.6.3" secao="6.6" slug="msrv-e-compatibilidade-do-toolchain">MSRV e compatibilidade do toolchain</subsecao>
    <subsecao id="6.6.4" secao="6.6" slug="estrategia-de-build-offline-do-sqlx">Estratégia de build offline do SQLx</subsecao>
    <secao id="7" slug="aplicacao-rust-decisao-e-limites" tipo="section">Aplicação Rust: decisão e limites</secao>
    <secao id="8" slug="motor-rust" tipo="section">Motor Rust</secao>
    <secao id="9" slug="yara-x" tipo="section">YARA-X</secao>
    <secao id="10" slug="parsing-evtx" tipo="section">Parsing EVTX</secao>
    <subsecao id="10.1.1" secao="10" slug="formatos-de-log-suportados">Formatos de log suportados</subsecao>
    <subsecao id="37.1" secao="37" slug="legibilidade-manutenibilidade-e-clareza-do-codigo">Legibilidade, manutenibilidade e clareza do código</subsecao>
    <subsecao id="37.2" secao="37" slug="responsabilidades-dos-diretorios-e-modulos">Responsabilidades dos diretórios e módulos</subsecao>
    <secao id="11" slug="pcap-e-memoria" tipo="section">PCAP e memória</secao>
    <secao id="12" slug="clamav" tipo="section">ClamAV</secao>
    <secao id="13" slug="nist-csf-2-0" tipo="section">NIST CSF 2.0</secao>
    <secao id="14" slug="mitre-att-ck" tipo="section">MITRE ATT&amp;CK</secao>
    <secao id="15" slug="analise-de-caminho-critico" tipo="section">Análise de caminho crítico</secao>
    <secao id="16" slug="matriz-de-impacto" tipo="section">Matriz de impacto</secao>
    <secao id="17" slug="ioc-extraction" tipo="section">IOC extraction</secao>
    <secao id="18" slug="sigma" tipo="section">Sigma</secao>
    <secao id="19" slug="preservacao-e-cadeia-de-custodia" tipo="section">Preservação e cadeia de custódia</secao>
    <secao id="20" slug="upload-arquivos-e-dfirpkg" tipo="section">Upload, arquivos e `.dfirpkg`</secao>
    <secao id="21" slug="privacidade-protecao-de-dados-e-retencao" tipo="section">Privacidade, proteção de dados e retenção</secao>
    <secao id="22" slug="modelo-de-dados" tipo="section">Modelo de dados</secao>
    <secao id="23" slug="rbac-e-seguranca-do-aplicativo" tipo="section">RBAC e segurança do aplicativo</secao>
    <secao id="24" slug="autenticacao-e-sessao" tipo="section">Autenticação e sessão</secao>
    <secao id="25" slug="self-hardening" tipo="section">Self-hardening</secao>
    <secao id="26" slug="task-queue" tipo="section">Task Queue</secao>
    <secao id="27" slug="persistencia" tipo="section">Persistência</secao>
    <secao id="28" slug="containerizacao-e-isolamento" tipo="section">Containerização e isolamento</secao>
    <secao id="29" slug="air-gapped" tipo="section">Air-Gapped</secao>
    <secao id="30" slug="relatorios" tipo="section">Relatórios</secao>
    <secao id="31" slug="assinatura-de-relatorios" tipo="section">Assinatura de relatórios</secao>
    <secao id="32" slug="hardening" tipo="section">Hardening</secao>
    <secao id="33" slug="criptografia-e-gestao-de-chaves" tipo="section">Criptografia e gestão de chaves</secao>
    <secao id="34" slug="observabilidade-e-auditoria" tipo="section">Observabilidade e auditoria</secao>
    <secao id="35" slug="testes" tipo="section">Testes</secao>
    <secao id="36" slug="performance" tipo="section">Performance</secao>
    <secao id="37" slug="estrutura-do-projeto" tipo="section">Estrutura do projeto</secao>
    <secao id="38" slug="roadmap" tipo="section">Roadmap</secao>
    <secao id="39" slug="criterios-de-aceitacao" tipo="section">Critérios de aceitação</secao>
    <secao id="40" slug="definition-of-done" tipo="section">Definition of Done</secao>
    <secao id="41" slug="licenciamento-e-sbom" tipo="section">Licenciamento e SBOM</secao>
    <secao id="42" slug="dataset-de-demonstracao" tipo="section">Dataset de demonstração</secao>
    <secao id="43" slug="apresentacao-de-5-minutos" tipo="section">Apresentação de 5 minutos</secao>
    <secao id="44" slug="plano-b" tipo="section">Plano B</secao>
    <secao id="45" slug="diretrizes-obrigatorias-para-a-llm-implementadora" tipo="section">Diretrizes obrigatórias para a LLM implementadora</secao>
    <secao id="46" slug="protocolo-de-implementacao" tipo="section">Protocolo de implementação</secao>
    <secao id="47" slug="protocolo-de-resposta-da-llm-durante-desenvolvimento" tipo="section">Protocolo de resposta da LLM durante desenvolvimento</secao>
    <secao id="48" slug="pendencias-externas-reais" tipo="section">Pendências externas reais</secao>
    <secao id="49" slug="metadados-da-demanda" tipo="section">Metadados da demanda</secao>
    <secao id="50" slug="decisoes-finais-consolidadas" tipo="section">Decisões finais consolidadas</secao>
    <secao id="51" slug="estado-do-documento" tipo="section">Estado do documento</secao>
    <secao id="52" slug="principio-final" tipo="section">Princípio final</secao>
    <secao id="53" slug="historico-de-revisoes" tipo="section">Histórico de revisões</secao>
    <secao id="apx-a" slug="guia-do-usuario-estrutura-minima" tipo="appendix">Guia do Usuário — estrutura mínima</secao>
    <secao id="apx-b" slug="compatibilidade-multiplataforma-windows-e-wsl2" tipo="appendix">Compatibilidade multiplataforma — Windows e WSL2</secao>
    <secao id="apx-c" slug="configuracao-do-ambiente-de-desenvolvimento" tipo="appendix">Configuração do ambiente de desenvolvimento</secao>
    <secao id="apx-d" slug="indice-normativo-para-consumo-por-llm" tipo="appendix">Índice normativo para consumo por LLM</secao>
    <secao id="apx-e" slug="catalogo-de-parsers-futuros" tipo="appendix">Catálogo de Parsers Futuros (Opcional)</secao>
    <secao id="apx-f" slug="modelo-de-consumo-por-llm-e-classes-de-runtime" tipo="appendix">Modelo de consumo por LLM e classes de runtime</secao>
  </secoes>
  <decisoes total="42">
    <decisao id="D01" secao="2">o cenário BEC será o principal cenário do dataset de demonstração, sem limitar a plataforma a BEC.</decisao>
    <decisao id="D02" secao="5">utilizar um monólito modular de implantação, com aplicação Rust e engine Rust em processos separados dentro do mesmo container no MVP.</decisao>
    <decisao id="D03" secao="5">cada invocação do engine roda dentro de um sandbox `bubblewrap` (`bwrap`) com namespaces dedicados, dentro do mesmo container.</decisao>
    <decisao id="D04" secao="6">utilizar Rust + Axum + Tokio no backend, Askama para templates e SQLx sobre SQLite, mantendo TypeScript + CSS próprio da aplicação no frontend.</decisao>
    <decisao id="D05" secao="8">o transporte de produção será stdin/stdout do subprocesso Rust, com protocolo JSONL versionado.</decisao>
    <decisao id="D06" secao="9">o MVP utilizará ruleset próprio e curado, versionado e com proveniência.</decisao>
    <decisao id="D07" secao="9">o MVP não realizará atualização automática de regras.</decisao>
    <decisao id="D08" secao="12">nenhuma fonte de assinatura de terceiros será padrão obrigatório do MVP.</decisao>
    <decisao id="D09" secao="19">assinatura digital da cadeia segue algoritmo/formato único definido pelo projeto.</decisao>
    <decisao id="D10" secao="24">aplicar backoff progressivo associado à conta e ao contexto de origem.</decisao>
    <decisao id="D11" secao="26">worker único no MVP.</decisao>
    <decisao id="D12" secao="26">retries limitados apenas para falhas potencialmente transitórias; falhas de integridade, schema incompatível, sandbox indisponível ou entrada inválida não devem ser repetidas indefinidamente.</decisao>
    <decisao id="D13" secao="26">quando o espaço livre atingir menos de 10% do volume operacional, novos uploads são rejeitados de forma controlada até recuperação do espaço.</decisao>
    <decisao id="D14" secao="27">não executar `VACUUM` ou `REINDEX` automaticamente por calendário fixo.</decisao>
    <decisao id="D15" secao="28">Podman rootless é o runtime de referência.</decisao>
    <decisao id="D16" secao="29">o ciclo de atualização é controlado e explícito.</decisao>
    <decisao id="D17" secao="30">Typst será o engine principal de PDF.</decisao>
    <decisao id="D18" secao="31">para o MVP, quando habilitada, a assinatura será destacada.</decisao>
    <decisao id="D19" secao="33">não introduzir SQLCipher como requisito do MVP.</decisao>
    <decisao id="D20" secao="36">os benchmarks devem ser reproduzíveis por comandos versionados; ferramentas de benchmark são dependências de desenvolvimento, não do produto.</decisao>
    <decisao id="D21" secao="40">releases do produto seguem SemVer `MAJOR.MINOR.PATCH`.</decisao>
    <decisao id="D22" secao="apx-a">existe um Guia do Usuário versionado junto com o software.</decisao>
    <decisao id="D23" secao="apx-b">Linux nativo é o alvo de implantação suportado.</decisao>
    <decisao id="D24" secao="apx-c">o workspace Rust e o `justfile` são a base do fluxo de desenvolvimento reproduzível.</decisao>
    <decisao id="D25" secao="apx-c">`cargo audit` é obrigatório antes de release e quando dependências Rust forem adicionadas ou atualizadas.</decisao>
    <decisao id="D26" secao="30">TypeScript é complementar ao HTML server-rendered para interações puramente locais; dados estruturados usam HTTP/JSON e SSE pode transportar eventos de jobs.</decisao>
    <decisao id="D27" secao="6">`tracing`/`tracing-subscriber` é a biblioteca de logging estruturado do projeto; nível controlado por `RUST_LOG`.</decisao>
    <decisao id="D28" secao="33">material de chave presente em memória do processo deve usar `zeroize` para zerar ao sair de escopo.</decisao>
    <decisao id="D29" secao="apx-c">quando CI com proteção de merge estiver configurado, todo push/PR deve rodar fmt/clippy/test/audit como gate obrigatório de merge; enquanto não houver workflow versionado no repositório, isso permanece política-alvo e os gates locais do `justfile`/release-gate são a autoridade operacional; bench/perf não bloqueiam PR.</decisao>
    <decisao id="D30" secao="41">distribuição fora do ambiente de desenvolvimento/demonstração exige validação jurídica formal do inventário de licenças como gate obrigatório.</decisao>
    <decisao id="D31" secao="6">cada release deve adotar somente versões estáveis mais recentes disponíveis no momento da seleção/atualização, fixar as versões exatas e registrar exceções de compatibilidade; `latest` não é identificador válido de release.</decisao>
    <decisao id="D32" secao="41">Hayabusa e Chainsaw são rejeitados como componentes, dependências, regras ou binários distribuídos do produto; qualquer reavaliação exige necessidade demonstrada, ADR, inventário de licença/proveniência e validação jurídica aplicável ao modelo de distribuição.</decisao>
    <decisao id="D33" secao="23">autorização de leitura deve ser combinada com classificação/visibilidade de dados; VIEWER é consulta mínima (`SUMMARY`), AUDITOR é fiscalização com `AUDIT_READ`/`AUDIT_VERIFY` e sem `EVIDENCE_READ`, ANALYST alcança `DETAILED` e ADMIN `RAW`; nenhuma concessão pode exceder o teto de visibilidade do papel.</decisao>
    <decisao id="D34" secao="6">Askama permanece nos templates web fixos; MiniJinja é adotado para CLI e templates deliberadamente dinâmicos, sem capacidades de filesystem, shell, rede ou execução privilegiada.</decisao>
    <decisao id="D35" secao="6">Tower/Tower-HTTP é a implementação preferencial de middleware HTTP genérico para request IDs, tracing, timeouts e headers de segurança, com features mínimas e sem CORS permissivo por padrão.</decisao>
    <decisao id="D36" secao="6">Supply chain usa `cargo-audit`, `cargo-deny`, `cargo-vet` e `cargo-cyclonedx`; `proptest` e `cargo-nextest` são ferramentas de teste; `AppConfig` centraliza configuração de processo e o runtime não lê `std::env` diretamente após o bootstrap.</decisao>
        <decisao id="D37" secao="6">Autenticação e fronteiras de erro usam tipos estruturados `thiserror`; falhas de renderização/readiness não podem ser ocultadas; propriedades críticas usam `proptest`; `cargo-nextest 0.9.140` é runner reprodutível sem retries no perfil padrão.</decisao>
    <decisao id="D38" secao="6">A aplicação não depende diretamente de `rand`/`rand_core` para geração de material aleatório. A fonte RNG direta é `getrandom 0.4.3`; Argon2 0.5.3 é usado com `default-features=false` e sem a feature `rand`, com salt gerado explicitamente pela aplicação e 16 bytes de entropia. Qualquer exceção futura exige revisão do grafo efetivo e teste de compatibilidade.</decisao>
    <decisao id="D39" secao="6">`getrandom 0.4.3` é a fronteira RNG direta do aplicativo com apenas a feature `std`; `sys_rng` permanece desabilitada para não importar `rand_core 0.10` sem necessidade. O grafo transitivo pode conter outras famílias RNG em ferramentas de teste, e isso não constitui violação enquanto não cruzar a fronteira de autenticação/segredos do aplicativo.</decisao>
  <decisao id="D40" secao="5/6">minimizar a superfície de runtime de produção: Rust no backend confiável; TypeScript somente como artefato frontend compilado; Deno como build/dev; CPython/Shell como tooling de engenharia/QA; sem Node/Bun/Deno no runtime.</decisao>
  <decisao id="D41" secao="30">usar HTTP/JSON para dados do browser e SSE para eventos unidirecionais de jobs quando necessário; WebSocket não faz parte do MVP.</decisao>
  <decisao id="D42" secao="30/35">Fase 0 utiliza renderer de timeline local mínimo sem dependência externa obrigatória; `vis-timeline`/D3 entram somente após necessidade demonstrada e validação de supply chain/offline.</decisao>
  </decisoes>
  <hipoteses total="1">
    <hipotese id="H01" secao="apx-c">Pode acelerar tempo de linkagem em builds Rust incrementais. HIPÓTESE não validada: consumo adicional estimado de ~200 MB de RAM durante o link — validar antes de assu...</hipotese>
  </hipoteses>
  <pendentes total="2">
    <pendente id="P01" secao="21">depende de validação com o cliente antes de virar requisito rígido — mas o modelo de dados não deve impedir essa flag de existir.</pendente>
    <pendente id="P02" secao="21">o prazo exato de retenção é decisão de negócio de cada organização usuária, não do software.</pendente>
  </pendentes>
  <requisitos-funcionais ref="secao-39">RF-001 a RF-047; RF-021 e RF-023 são critérios de Fase 4 e não bloqueiam o MVP.</requisitos-funcionais>
  <modelo-semantico versao="1.0">
    <tipo id="requirement" autoridade="normativa">Requisito funcional ou não funcional que pode ser rastreado.</tipo>
    <tipo id="decision" autoridade="arquitetural">Decisão de engenharia registrada no corpo normativo.</tipo>
    <tipo id="constraint" autoridade="normativa">Restrição que limita implementação, operação ou distribuição.</tipo>
    <tipo id="acceptance-criterion" autoridade="normativa">Condição objetiva para considerar uma capacidade atendida.</tipo>
    <tipo id="procedure" autoridade="operacional">Sequência controlada de execução, validação ou atualização.</tipo>
    <tipo id="llm-rule" autoridade="normativa-para-implementacao">Regra explícita para comportamento da LLM implementadora.</tipo>
    <tipo id="gate" autoridade="verificacao">Verificação que produz evidência de conformidade.</tipo>
    <tipo id="hypothesis" autoridade="nao-normativa">Hipótese que não pode ser tratada como requisito.</tipo>
    <tipo id="pending" autoridade="nao-normativa">Questão ainda dependente de decisão ou informação externa.</tipo>
    <tipo id="rejected" autoridade="historica">Sugestão avaliada e formalmente não incorporada.</tipo>
  </modelo-semantico>
</indice-normativo>
```


# Apêndice E. Catálogo de Parsers Futuros (Opcional)
<!-- @section id="apx-e" slug="catalogo-de-parsers-futuros" -->

Esta seção registra **candidatos de evolução**, não compromissos de implementação. A presença de uma tecnologia neste catálogo não significa que ela será incorporada ao produto. Cada candidato deve passar por avaliação independente segundo o **Princípio de Necessidade Demonstrada**, segurança, qualidade do parser, desempenho, manutenção, dependências, compatibilidade com o modo offline e licenciamento.

As licenças abaixo são **informativas e devem ser revalidadas na versão exata do artefato antes da incorporação**. Uma licença do projeto/agregador não deve ser presumida como licença de todos os componentes, regras ou dados que ele distribui.

## E.1. Alta prioridade (Fases 2–3)

| **6.1.0** | **Política de versões mais recentes e reprodutibilidade** | **Regra explícita de que o projeto deve adotar somente versões estáveis mais recentes disponíveis no momento da seleção/atualização de cada release, sem uso de versões EOL/experimentais quando houver alternativa estável compatível. As versões efetivas continuam fixadas por lockfile/manifesto, checksum/digest e SBOM; `latest` é proibido como identificador de release. Foi adicionada exceção formal para incompatibilidades justificadas, com registro e reavaliação. RF-047 e D31 adicionados; índice normativo atualizado. Nenhuma decisão arquitetural estrutural foi alterada.** |
| Formato/artefato | Descrição | Candidato | Licença/status | Observação |
|---|---|---|---|---|
| Registry Hives | Hives do Registro do Windows | `nt_hive2` | GPL-3.0 | Tecnicamente relevante, mas a licença copyleft exige avaliação antes de linkagem direta ao engine. [26] |
| Syslog | Mensagens RFC 5424; RFC 3164 requer tratamento adicional | `syslog_proto` ou parser próprio | Licença a verificar na versão adotada | O crate consultado implementa RFC 5424 e é experimental; não deve ser tratado como escolha fechada. [27] |
| PCAP/PCAPNG | Estruturas de captura e parsing de pacotes | `pcap-parser` | MIT OR Apache-2.0 | Bom candidato para parsing de contêineres PCAP/PCAPNG; reassembly e protocolos de aplicação permanecem trabalho adicional. [28] |

## E.2. Média prioridade (Fases 3–4)

| Formato/artefato | Descrição | Candidato | Licença/status | Observação |
|---|---|---|---|---|
| MFT / NTFS | Master File Table e artefatos NTFS | `forensic-rs` / candidatos especializados | Licença a verificar na versão adotada | `chainsaw` possui suporte a MFT, mas o projeto é GPL-3.0; deve ser tratado como ferramenta candidata, não dependência automática do engine. [29] |
| Apache / Nginx | Logs de acesso HTTP | Parser próprio ou biblioteca especializada | A verificar | A gramática desses logs é simples o suficiente para permitir implementação interna quando houver necessidade demonstrada; evitar dependência apenas por conveniência. |
| Prefetch | Arquivos `.pf` do Windows | `frnsc-prefetch` ou parser próprio | MIT para `frnsc-prefetch`; verificar versão exata | Existe implementação Rust específica para Prefetch; ainda assim, a adoção depende de necessidade demonstrada e validação de manutenção/desempenho. [34] |
| JSONL / CSV | Logs estruturados | parser próprio / `pandoraslogs` como referência | A verificar | Formatos genéricos devem priorizar parser interno pequeno e determinístico antes de adicionar uma dependência externa. [31] |

## E.3. Baixa prioridade (Fase 4+)

| Formato/artefato | Descrição | Candidato | Licença/status | Observação |
|---|---|---|---|---|
| AmCache | Cache de execução/aplicações do Windows | AmcacheParser / implementação própria | MIT para o repositório de Eric Zimmerman; verificar versão exata | Ferramenta externa pode ser usada como referência ou integração futura; não constitui dependência do MVP. [32] |
| macOS Unified Logs | Logs unificados do macOS | `macos-UnifiedLogs` / parser especializado | A verificar | Relevante apenas se surgir requisito de suporte a macOS; não deve ser adicionado por antecipação. [33] |
| PE Files | Metadados e estrutura de executáveis Windows | `exhume_artefacts` ou `goblin` | A verificar na versão adotada | O `exhume_artefacts` atualmente possui parser de PE; a necessidade deve ser avaliada contra a funcionalidade YARA-X/IOC existente. [30] |

## E.4. Critérios de incorporação

Antes de adicionar um parser ao produto, registrar pelo menos:

- caso de uso concreto e requisito que o justifica;
- formatos e versões efetivamente suportados;
- comportamento diante de entrada malformada ou adversarial;
- limites de CPU, memória, tempo e tamanho de entrada;
- estratégia de atualização e manutenção;
- licença do componente e de suas dependências transitivas;
- impacto no SBOM e no processo de release;
- impacto no modo offline/Air-Gapped;
- benchmark representativo, quando o parser for de processamento pesado;
- testes de integridade e regressão com corpus versionado;
- ADR registrando a decisão de adoção ou rejeição.

**REGRA:** a existência de um parser candidato neste apêndice não autoriza sua incorporação automática. O catálogo é informativo e deve ser revalidado antes de cada decisão de implementação.

## E.5. Licenciamento e integração

A licença de uma biblioteca não determina, isoladamente, a licença ou as obrigações de todos os dados, regras, modelos ou ferramentas que possam acompanhá-la. Na análise de uma futura integração, devem ser distinguidos:

1. biblioteca linkada ao engine;
2. executável externo chamado como subprocesso;
3. regras/datasets distribuídos separadamente;
4. dependências transitivas;
5. código copiado ou adaptado para o projeto.

A integração final deve passar pelo inventário técnico da seção 41 e ser registrada em ADR conforme a seção 40.2. O inventário técnico de licenças não constitui parecer jurídico.

# Apêndice F. Modelo de consumo por LLM e classes de runtime
<!-- @section id="apx-f" slug="modelo-de-consumo-por-llm-e-classes-de-runtime" -->

Este apêndice define **como uma LLM deve interpretar a especificação**, não substitui o corpo normativo. Seu objetivo é tornar a documentação mais determinística para agentes de implementação sem converter todo o documento para XML.

## F.1. Princípio de representação
## F.1.1. Classes de runtime e tooling

```xml
<runtime-model version="1">
  <class id="production-runtime">Rust application + Rust engine + static web assets</class>
  <class id="build-toolchain">Rust/Cargo + Deno</class>
  <class id="engineering-tooling">CPython + Shell + fixtures + benchmarks + release tools</class>
</runtime-model>
```

**Regra:** a existência de uma linguagem no build ou no tooling não implica que o runtime de produção deva contê-la.



- Markdown é a fonte canônica e humana.
- XML é uma camada estruturada derivada, para indexação, regras, decisões, gates e rastreabilidade.
- IDs como `RF-*`, `D*`, `LLM-*`, `CA-*`, `TEST-*`, `GATE-*`, `H-*`, `P-*` e `ADR-*` devem ser estáveis dentro da versão documental em que foram publicados.
- Uma LLM nunca deve inferir autoridade apenas da presença de uma tag; o tipo semântico e a seção de origem importam.

## F.2. Tipos semânticos

| Tipo | Significado | Pode virar requisito automaticamente? | Evidência esperada |
|---|---|---:|---|
| `requirement` | requisito rastreável | não, sem fonte | demanda + seção normativa |
| `decision` | escolha arquitetural/engenharia | não | decisão + justificativa |
| `constraint` | restrição | não | fonte/justificativa |
| `acceptance-criterion` | condição objetiva | não | teste ou verificação correspondente |
| `procedure` | procedimento de execução | não | comando/registro/resultado |
| `llm-rule` | regra para a LLM implementadora | não | inspeção ou gate aplicável |
| `gate` | controle de verificação | não | saída/evidência do gate |
| `hypothesis` | hipótese não confirmada | **não** | benchmark, teste ou fonte futura |
| `pending` | pendência | **não** | decisão/informação externa |
| `rejected` | sugestão rejeitada | **não** | justificativa/ADR quando aplicável |

## F.3. Contrato mínimo de rastreabilidade

Para capacidades relevantes, a cadeia ideal é:

```text
requirement
    ↓
decision / constraint
    ↓
implementation
    ↓
acceptance-criterion
    ↓
test / benchmark
    ↓
gate
    ↓
evidence
    ↓
status
```

A ausência de um elo deve ser registrada como lacuna, e não preenchida por inferência. Um arquivo ou função pode ser citado como implementação, mas não como prova de integração ou verificação.

## F.3.1. Contrato de frontend e transporte

```xml
<frontend-contract>
  <server-rendering>HTML server-rendered</server-rendering>
  <rich-ui>TypeScript compiled to browser JavaScript</rich-ui>
  <data>HTTP/JSON</data>
  <job-events>SSE when necessary</job-events>
  <websocket>forbidden in MVP unless new decision/evidence</websocket>
  <node-runtime>forbidden in production image</node-runtime>
  <bun-runtime>forbidden in production image</bun-runtime>
  <deno-runtime>forbidden in production image</deno-runtime>
</frontend-contract>
```

## F.4. Regras de interpretação para LLM

```xml
<llm-consumption-contract version="1.0">
  <authority>
    <layer rank="1">requirements-explicit-in-demand</layer>
    <layer rank="2">normative-body-of-specification</layer>
    <layer rank="3">stable-ids-and-decisions</layer>
    <layer rank="4">derived-indexes-and-xml</layer>
    <layer rank="5">examples-hypotheses-and-auxiliary-text</layer>
  </authority>
  <rules>
    <rule id="LLM-C-001">Nunca elevar hipótese, exemplo ou sugestão a requisito.</rule>
    <rule id="LLM-C-002">Nunca declarar um gate como passado sem evidência de execução.</rule>
    <rule id="LLM-C-003">Nunca tratar índice ou bloco XML derivado como autoridade superior ao corpo.</rule>
    <rule id="LLM-C-004">Quando houver conflito, preservar o requisito de maior precedência e registrar a divergência.</rule>
    <rule id="LLM-C-005">Ao alterar contrato, persistência, segurança ou arquitetura, atualizar os artefatos de rastreabilidade afetados.</rule>
    <rule id="LLM-C-006">Quando uma lacuna puder alterar uma decisão de engenharia ou requisito, marcar como pending em vez de inventar uma resposta.</rule>
  </rules>
</llm-consumption-contract>
```

## F.5. Prompt de continuidade

O arquivo `PROMPT_NOVA_CONVERSA.md` deve permanecer curto o suficiente para ser usado como prompt operacional, mas deve apontar para esta especificação e para os artefatos de estado. Ele pode usar pequenos blocos XML para declarar autoridade, regras e gates, sem exigir a duplicação da especificação completa.

## F.6. Regra de manutenção

Quando uma seção normativa, decisão, requisito ou gate for criado, removido ou renomeado, atualizar na mesma mudança:

1. o corpo Markdown;
2. o índice XML;
3. o RTM, quando aplicável;
4. o status/changelog;
5. o prompt de continuidade, quando a mudança afetar o comportamento esperado da LLM.

O índice é, portanto, um **artefato regenerável**, enquanto o corpo normativo permanece a fonte primária.


## F.7. Manifesto de autoridade documental

Para evitar conflitos entre arquivos que descrevem o mesmo projeto, a autoridade é definida por **tipo de informação**, e não por proximidade do arquivo com o código. O manifesto abaixo é a regra operacional: artefatos derivados devem apontar para sua fonte primária, e não competir com ela.

```xml
<document-authority version="1.0">
  <domain name="business-requirements">
    <authority>demanda SAGA SENAI 12412</authority>
    <secondary>corpo normativo desta especificação</secondary>
  </domain>
  <domain name="architecture">
    <authority>decisões e seções normativas desta especificação</authority>
    <secondary>ADRs que aprofundam decisões já existentes</secondary>
  </domain>
  <domain name="implementation-status">
    <authority>docs/FASE0_STATUS.md</authority>
    <secondary>RTM + evidências de execução</secondary>
  </domain>
  <domain name="traceability">
    <authority>docs/rtm.md</authority>
    <secondary>IDs e evidências referenciadas nos artefatos</secondary>
  </domain>
  <domain name="dependencies">
    <authority>Cargo.lock + cargo metadata --locked</authority>
    <secondary>Cargo.toml e SBOM gerado</secondary>
  </domain>
  <domain name="release-verification">
    <authority>docs/RELEASE_GATE.md + evidências dos comandos executados</authority>
  </domain>
  <domain name="llm-operational-rules">
    <authority>AGENTS.md + seção 45 + Apêndice F</authority>
    <secondary>PROMPT_NOVA_CONVERSA.md</secondary>
  </domain>
</document-authority>
```

**Regra:** quando dois artefatos de um mesmo domínio divergirem, a LLM deve identificar a fonte de autoridade, registrar a inconsistência e corrigir o artefato derivado. Não escolher a versão “mais conveniente”.

## F.8. Ciclo de estado de um requisito

Para evitar que “há código” seja confundido com “capacidade concluída”, o estado de uma capacidade deve seguir, quando aplicável:

```text
PLANNED
   ↓
IMPLEMENTED
   ↓
INTEGRATED
   ↓
VERIFIED
   ↓
ACCEPTED
```

- **PLANNED:** requisito ou decisão identificado, sem implementação demonstrada.
- **IMPLEMENTED:** existe implementação identificável.
- **INTEGRATED:** existe um caminho executável que alcança a capacidade pela fronteira prevista de produção.
- **VERIFIED:** teste, benchmark ou gate correspondente foi realmente executado e produziu evidência suficiente.
- **ACCEPTED:** os critérios de aceitação aplicáveis estão satisfeitos e não existe pendência bloqueadora conhecida.

`WAIVED` é uma exceção de governança a um gate, não um estado de verificação. `PENDING`, `BLOCKED` e `STALE` impedem que uma capacidade ou transição seja apresentada como verificada.

Estados podem possuir qualificadores, por exemplo `IMPLEMENTED / BUILD-PENDING`, `INTEGRATED / RUNTIME-PENDING` ou `VERIFIED / EVIDENCE-INCOMPLETE`. Esses qualificadores não devem ser omitidos para produzir uma aparência artificial de conclusão.

### F.8.1. Staleness da evidência

Uma evidência anteriormente válida torna-se `STALE` quando muda qualquer condição relevante para o gate: commit/tree de código, `Cargo.lock`, SBOM, toolchain, imagem/container, configuração de runtime, fixture/dataset ou parâmetro relevante. A evidência stale não deve ser reaproveitada como prova do novo estado.

### F.8.2. Identidade e substituição de IDs

IDs normativos são imutáveis e não devem ser reutilizados. Quando uma decisão, requisito ou contrato for substituído, o novo registro deve usar um novo ID e apontar para o anterior com `supersedes`.

## F.9. Contrato de evidência para gates

Um gate importante só pode ser declarado `PASS` quando houver evidência identificável. A evidência deve ser suficientemente específica para permitir repetição ou auditoria posterior.

```xml
<gate-evidence>
  <evidence id="EVD-*">
    <gate ref="GATE-*"/>
    <status>PASS|FAIL|PENDING|BLOCKED|WAIVED|STALE</status>
    <executed-at>timestamp</executed-at>
    <commit-tree>commit ou hash da árvore testada</commit-tree>
    <toolchain>rustc/cargo e versão relevante</toolchain>
    <environment>host/container/configuração quando relevante</environment>
    <command>comando ou procedimento executado</command>
    <artifact>arquivo/log/hash/fixture quando aplicável</artifact>
    <result>resultado resumido e objetivo</result>
  </evidence>
</gate-evidence>
```

**Regra:** texto como “deve passar”, “está correto” ou “foi validado” não substitui evidência de execução. Quando o ambiente não permitir a execução, o gate permanece `PENDING`.

## F.10. Controle de mudanças documentais

Uma mudança deve ser classificada antes de ser aplicada:

| Classe | Exemplo | Atualizações mínimas |
|---|---|---|
| `DOC-ONLY` | explicação ou correção editorial sem efeito normativo | changelog + revisão documental |
| `NORMATIVE` | novo requisito, restrição, critério ou regra LLM | especificação + índice XML + RTM + prompt/AGENTS se aplicável |
| `DECISION` | alteração de decisão arquitetural | especificação + ADR + RTM + changelog + prompt/AGENTS |
| `CONTRACT` | mudança em protocolo, schema, persistência ou interface | especificação + implementação + testes + RTM + gate afetado |
| `DEPENDENCY` | inclusão/remoção/atualização de dependência | manifesto + lockfile + SBOM + supply-chain + testes |
| `RELEASE` | mudança que afeta bundle, imagem ou distribuição | release gate + SBOM + hashes/digests + changelog |

**Regra:** toda alteração deve declarar o tipo da mudança e os artefatos afetados. Se a mudança não puder ser classificada sem ambiguidade, tratá-la como `NORMATIVE` até revisão humana.

## F.11. Marcação normativa e explicativa

Quando a redação puder ser confundida com uma obrigação, a seção deve deixar explícito o papel do texto:

- **NORMATIVO:** define o que deve ser feito ou evitado.
- **RATIONALE:** explica por que a decisão existe.
- **CRITÉRIO DE ACEITAÇÃO:** define uma condição observável.
- **PROCEDIMENTO:** define como verificar ou executar.
- **EXEMPLO:** ilustra, mas não cria obrigação.
- **NOTA:** esclarece sem alterar a regra.

A existência de exemplos ou racionales não amplia o escopo funcional e não pode ser usada isoladamente como justificativa para implementação.

## F.12. Definition of Ready para a próxima fase

Antes de iniciar uma nova fase de implementação, a fase anterior deve demonstrar, no mínimo:

1. arquitetura e contratos normativos sem divergências conhecidas;
2. dependências, lockfile e SBOM reconciliados;
3. RTM sem lacunas críticas de rastreabilidade;
4. gates obrigatórios em `PASS` ou dispensados por `WVR-*` formal;
5. fixtures/datasets necessários para a próxima etapa identificados;
6. ambiente de execução e toolchain registrados;
7. riscos e exceções conhecidos documentados;
8. ADRs necessárias atualizadas;
9. documentação operacional e prompt de continuidade coerentes com a versão vigente.

Isso é um **Definition of Ready**, não uma exigência de concluir funcionalidades futuras antecipadamente.

## F.13. Classificação de fatos e incerteza

A LLM deve diferenciar, quando isso puder afetar a implementação ou uma alegação de verificação:

- `REPO-FACT`: fato diretamente observável nos arquivos/artefatos do repositório;
- `EXTERNAL-FACT`: fato externo que exige fonte ou verificação independente;
- `ASSUMPTION`: hipótese de trabalho ainda não confirmada;
- `INFERENCE`: conclusão derivada de fatos, mas não diretamente observada;
- `PENDING-HUMAN`: decisão ou informação que depende de autoridade humana.

Nunca apresentar `ASSUMPTION` ou `INFERENCE` como fato implementado/verificado.

## F.14. Waivers e exceções

Uma exceção a gate obrigatório deve possuir ID `WVR-*` e registrar, no mínimo:

```xml
<waiver id="WVR-*" gate="GATE-*" status="approved">
  <reason>motivo concreto</reason>
  <scope>escopo da exceção</scope>
  <approved-by>responsável com autoridade</approved-by>
  <approved-at>timestamp</approved-at>
  <valid-until>timestamp ou omitted quando não aplicável</valid-until>
</waiver>
```

`WAIVED` não deve ser usado para mascarar ausência de testes por conveniência. A exceção precisa ser limitada ao gate e condição descritos, e qualquer alteração relevante no artefato ou no contexto deve provocar reavaliação.

## F.15. Regra de fatos observáveis e alegações externas

A LLM não deve converter um fato de documentação externa em fato do repositório sem evidência local. Sempre que a afirmação puder ser relevante para uma decisão, identificar sua natureza como `REPO-FACT`, `EXTERNAL-FACT`, `ASSUMPTION`, `INFERENCE` ou `PENDING-HUMAN`.

Exemplo: “a versão estável atual é X” é um fato externo; “o projeto foi testado com X” exige evidência de execução do ambiente.

## F.16. Catálogo mínimo de anti-padrões para LLM

As seguintes ações são proibidas salvo alteração normativa/ADR que as autorize:

```xml
<anti-patterns>
  <item id="AP-001">implementar fase futura por antecipação</item>
  <item id="AP-002">trocar biblioteca por familiaridade da LLM</item>
  <item id="AP-003">declarar gate aprovado sem evidência</item>
  <item id="AP-004">tratar existência de arquivo/função como integração</item>
  <item id="AP-005">usar índice/XML derivado para sobrescrever o corpo normativo</item>
  <item id="AP-006">corrigir teste removendo a condição que ele deveria detectar</item>
  <item id="AP-007">introduzir dependência ou abstração sem necessidade demonstrada</item>
  <item id="AP-008">reabrir decisão rejeitada sem registrar nova evidência</item>
  <item id="AP-009">reutilizar ID normativo de requisito, decisão, teste ou gate</item>
  <item id="AP-010">usar evidência `STALE` como prova de estado atual</item>
  <item id="AP-011">tratar `WAIVED` como equivalente a `VERIFIED`</item>
</anti-patterns>
```

## F.17. Mapa resumido de decisões reavaliáveis

Decisões fechadas não são imutáveis, mas também não devem ser reabertas por preferência. Uma reavaliação requer evidência do tipo indicado abaixo:

| Decisão | Evidência que pode justificar reavaliação | Registro mínimo |
|---|---|---|
| monólito modular + engine separado | escala/concurrency ou threat model diferente | benchmark/ameaça + ADR |
| SQLite + WAL | múltiplos escritores ou necessidade de escala horizontal real | benchmark/caso operacional + ADR |
| JSONL stdin/stdout | necessidade de execução remota/distribuída | requisito + benchmark + ADR |
| bubblewrap por job | teste adversarial ou threat model exigir fronteira adicional | teste/ameaça + ADR |
| getrandom na fronteira RNG | incompatibilidade comprovada ou requisito criptográfico novo | grafo/API/teste + ADR |
| YARA-X + ruleset curado | requisito de compatibilidade ou necessidade concreta de outra engine | benchmark/licença/requisito + ADR |
| backend confiável em Rust | benchmark/requisito demonstrar runtime de servidor superior ou necessidade operacional | benchmark + threat model + ADR |

**Regra:** “há uma alternativa mais moderna” não é, sozinho, critério suficiente para reabrir uma decisão.

