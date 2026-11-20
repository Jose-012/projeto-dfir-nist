# 40 — Assinatura destacada e verificação dos entregáveis

**What to build:** Assinatura Ed25519 do SHA-256 do `manifest.json`, artefato `manifest.json.sig`, verificação CLI e auditoria, com chave privada externa e preview não assinado.

**Blocked by:** F1-07 (implementado); SVG quando incluído no mesmo manifest.

**Status:** PLANNED (ativado pelo operador em 2026-11-20, ADR-0030 D-1/D-2).

**Origem:** RF-013, RF-015, RF-038 e seção 31; melhoria de engenharia, não requisito novo da demanda.

- [ ] Formato/versionamento da assinatura e fingerprint público são documentados e determinísticos.
- [ ] Chave privada é validada fora do storage interno/exportação e nunca aparece em log, manifest ou auditoria.
- [ ] Gerar com assinatura verifica antes de auditar; preview não cria nem assina arquivos.
- [ ] CLI detecta documento/manifest/assinatura alterados, chave errada e artefato ausente.
- [ ] Dependência passa por `just supply-chain`; testes, E2E e documentação não alegam validade jurídica.
