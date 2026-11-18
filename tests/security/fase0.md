# Fase 0 - segurança do produto local

A linha de base de segurança é construída em torno da fronteira forense, não de autenticação de usuários.

## Controles verificados

- aplicação local com bind em loopback por padrão;
- exposição não-loopback exige opt-in explícito e é destinada somente a ambientes de implantação explicitamente isolados definidos pelo operador;
- evidências são tratadas como entrada não confiável;
- `dfir-engine` executa em processo separado;
- parser executa em processo separado;
- limites de linha, itens, saída, timeout e concorrência;
- staging separado;
- audit chain append-only e tamper-evident;
- BLAKE3-256-v2-JCS para novos eventos (RFC 8785), com verificação de BLAKE3-256-v1; SHA-256 permanece apenas nos fluxos de evidência explicitamente requeridos;
- origem de evidência permanece separada do processo de parsing.
