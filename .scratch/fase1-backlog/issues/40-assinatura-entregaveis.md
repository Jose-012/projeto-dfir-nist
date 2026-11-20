# 40 — Assinatura destacada e verificação dos entregáveis

**What to build:** Assinatura Ed25519 do SHA-256 do `manifest.json`, artefato `manifest.json.sig`, verificação CLI e auditoria, com chave privada externa e preview não assinado.

**Blocked by:** F1-07 (implementado); SVG quando incluído no mesmo manifest.

**Status:** VERIFIED em Linux x86_64 (2026-11-20, ADR-0030 D-1/D-2, `EVD-056`). Não é declaração de validade jurídica nem aceite externo do release.

**Origem:** RF-013, RF-015, RF-038 e seção 31; melhoria de engenharia, não requisito novo da demanda.

- [x] Formato/versionamento da assinatura e fingerprint público são documentados e determinísticos.
- [x] Chave privada é validada fora do storage interno/exportação e nunca aparece em log, manifest ou auditoria.
- [x] Gerar com assinatura verifica antes de auditar; preview não cria nem assina arquivos.
- [x] CLI detecta documento/manifest/assinatura alterados, chave errada e artefato ausente.
- [x] Dependência passa por `just supply-chain`; testes, E2E e documentação não alegam validade jurídica.

**Evidência:** `EVD-056`; cinco testes de signing, um de geração assinada, supply-chain com 621 dependências e E2E 123/123. A chave usada foi sintética/temporária e a raiz do E2E foi removida.
