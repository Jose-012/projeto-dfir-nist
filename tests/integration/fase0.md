# Fase 0 — testes de integração

Os testes abaixo são executáveis ou apontam explicitamente para o mecanismo executável correspondente.

## Gate 1 — compilação

```bash
just fmt
just check
```

Critério: workspace `dfir-app` + `dfir-engine` compila com `--locked` e sem alterações de formatação.

## Gate 2 — banco/migrations

```bash
just test
```

Inclui migration e testes da cadeia de auditoria. O teste concorrente de auditoria dispara múltiplos appenders e exige `verify() == true`.

## Gate 3 — caminho real do engine

O engine deve usar `evtx 0.12.2` como parser EVTX; o teste deve registrar uma fixture real e validar que eventos e erros de registro atravessam o protocolo `dfir-protocol`.

```bash
just runtime-test
```

No Windows, `scripts/run_runtime_test.py` inicia o PowerShell do teste nativo
com `-ExecutionPolicy Bypass` **somente no processo filho**. Isso evita falhas do
`AuthorizationManager` em hosts não interativos/sem a validação de zona do Shell,
sem alterar a Execution Policy global do usuário ou da máquina. O bypass é restrito
a este script fixo de QA e não é usado pelo runtime do produto.


## Gate 4 — segurança de configuração e processo

```bash
just runtime-security
```

Inclui verificação da configuração local, fronteira de processo, escopo de caminhos e rejeição de ferramentas externas.

## Critério

A Fase 0 não é considerada validada apenas porque uma função ou script existe. Cada gate deve ser executado no ambiente correspondente e registrado no status da fase.

### Invariante do protocolo de processo

No modo nativo, o processo pai deve fechar explicitamente sua extremidade de `stdin`
após enviar a requisição JSONL. O engine usa EOF para encerrar o fluxo de entrada; manter
o pipe aberto causa espera mútua entre pai e filho e deve ser tratado como regressão do
gate de runtime.
