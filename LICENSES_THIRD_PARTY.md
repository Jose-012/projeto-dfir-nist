<!-- doc-type: reference | status: current -->
# Third-party licenses

As dependências devem ser inventariadas a partir do `Cargo.lock` e do inventário de componentes do release. Este arquivo é o inventário de licenças do pacote; a conferência jurídica antes de distribuir é a descrita na seção 41.3 da especificação.

> **Gap origem: AUS-04 (2026-11-04); licenças preenchidas em 2026-11-09 e atualizadas em 2026-11-14.** Inventário gerado por `cargo metadata --locked --format-version 1` (cargo 1.99.1, rustc 1.99.1) sobre o `Cargo.lock` deste pacote (SHA-256 `e218752099c595d60161dcc1b8473770310396a0674381ec9c68ac3c7810e57a`): 612 pacotes, 3 do workspace e 609 de terceiros; a metadata e o lock têm exatamente os mesmos pacotes. A licença de cada pacote é o campo `license` declarado no manifesto do próprio crate; o texto dos arquivos de licença de cada crate não foi revisado. SBOM não faz parte do escopo (ADR-0028). A atualização de 2026-11-14 (tarde) acompanha o upgrade `eframe`/`egui`/`egui_extras`/`egui_kittest` 0.37.0 + `rfd` 0.18.0 com `eframe` sem `accesskit` (removeu do grafo a cadeia `zbus`/`zvariant`/`atspi`/`ashpd`/`sctk-adwaita`/`ttf-parser`), o `cargo update` geral (`jiff 0.2.44`, `toml 1.2.0`, `uuid 1.29.2` sem `defmt`) e, na rodada da manhã, `evtx 0.15.1` (removeu `encoding` e a árvore do `skeptic`; entrou `encoding_rs`) e o `async-executor 1.14.1`.

> **Atualização 2026-11-18 (escopo de `proptest`):** a dev-dependency saiu de `dfir-app` e passou a `dfir-protocol`, sem adicionar, remover ou atualizar pacotes. O lock continua com 612 pacotes e o inventário de licenças permanece válido; novo SHA-256 do `Cargo.lock`: `07dafb055295648969544e7fd5fd84b37e8830a565314e4175afc9696572fa95`.

> **Atualização 2026-11-19 (segurança):** inventário regenerado por `cargo metadata --locked --format-version 1` com cargo/rustc 1.100.0: 613 pacotes, 3 do workspace e 610 de terceiros; SHA-256 do `Cargo.lock` `bf5c5f5fef7c83af6846085681b5ee9587285df68f280e7a29d75d9fd1be4a14`. Atualizações mínimas: `sqlx 0.9.1`, `libsqlite3-sys 0.40.0`, `wasmtime 48.0.9` e a família Cranelift 0.135.9; entrou `objc2-system-configuration 0.3.2`. SBOM permanece fora do escopo (ADR-0028).

> **Atualização 2026-11-20 (F1-40):** inventário regenerado por `cargo metadata --locked --format-version 1` com cargo/rustc 1.100.0: 621 pacotes, 3 do workspace e 618 de terceiros; SHA-256 do `Cargo.lock` `09116593a6ccba436852821fbbfc09aa55b55a7fefb1784be376587256db2304`. A mudança adiciona `ed25519-dalek 3.0.0` e sua árvore de assinatura Ed25519. Metadata e lock têm as mesmas identidades de pacote. SBOM permanece fora do escopo (ADR-0028).

## Ferramentas explicitamente rejeitadas como componentes do produto

**Hayabusa** e **Chainsaw** não fazem parte do produto e não devem ser adicionados a `Cargo.toml`, inventário de componentes, fixtures ou pacotes distribuídos sem uma nova decisão arquitetural e revisão jurídica aplicável ao modelo efetivo de distribuição.

A razão técnica registrada na ADR-0002 é preservar a possibilidade de distribuição comercial/licenciamento próprio, reduzir supply-chain e evitar incorporar regras/artefatos de terceiros sem necessidade demonstrada.

## Dependências diretas já inventariadas (licenças da `cargo metadata`)

- serde_json_canonicalizer 0.3.2 — MIT.
- thiserror 2.0.21 — MIT OR Apache-2.0.
- ed25519-dalek 3.0.0 — BSD-3-Clause.
- proptest 1.12.0 — MIT OR Apache-2.0.
- evtx 0.15.1 — MIT/Apache-2.0.
- eframe 0.37.0 — MIT OR Apache-2.0.
- egui 0.37.0 — MIT OR Apache-2.0.
- egui_extras 0.37.0 — MIT OR Apache-2.0.
- rfd 0.18.0 — MIT.
- windows-sys 0.61.2 — MIT OR Apache-2.0.
- dfir-protocol 0.1.0 — código próprio, Apache-2.0 (`Cargo.toml`); `dfir-app` e `dfir-engine` idem.
- cargo-audit 0.22.2 — ferramenta de desenvolvimento/release, fora do lock e do runtime; licença não verificada por `cargo metadata`.

## Resumo por licença

Nenhum pacote declara apenas uma licença copyleft (GPL, LGPL, AGPL, MPL, EPL, CDDL): as únicas expressões que citam uma delas oferecem alternativa permissiva, e são `r-efi` 5.3.0 e 6.0.0 (`MIT OR Apache-2.0 OR LGPL-2.1-or-later`) e `self_cell` 1.3.0 (`Apache-2.0 OR GPL-2.0-only`): em ambas a escolha é do licenciado, e usa-se a opção permissiva. `epaint_default_fonts` traz fontes (`OFL-1.1` e `Ubuntu-font-1.0`), que exigem manter os avisos de atribuição ao distribuir o binário.

| Licença (expressão SPDX declarada) | Pacotes |
|---|---|
| MIT OR Apache-2.0 | 276 |
| MIT | 123 |
| Apache-2.0 OR MIT | 43 |
| MIT/Apache-2.0 | 29 |
| Apache-2.0 WITH LLVM-exception | 27 |
| Apache-2.0 | 18 |
| Unicode-3.0 | 18 |
| Unlicense OR MIT | 13 |
| Zlib OR Apache-2.0 OR MIT | 13 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 12 |
| BSD-3-Clause | 6 |
| MIT OR Apache-2.0 OR Zlib | 6 |
| Zlib | 4 |
| Apache-2.0/MIT | 3 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 2 |
| BSD-2-Clause OR Apache-2.0 OR MIT | 2 |
| BSD-3-Clause OR Apache-2.0 | 2 |
| BSD-3-Clause OR MIT OR Apache-2.0 | 2 |
| BSL-1.0 | 2 |
| MIT OR Apache-2.0 OR LGPL-2.1-or-later | 2 |
| MIT OR Zlib OR Apache-2.0 | 2 |
| Unlicense/MIT | 2 |
| (Apache-2.0 OR MIT) AND BSD-3-Clause | 1 |
| (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 | 1 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| Apache-2.0 / MIT | 1 |
| Apache-2.0 AND MIT | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR BSL-1.0 OR MIT | 1 |
| Apache-2.0 OR GPL-2.0-only | 1 |
| BSD-2-Clause OR MIT OR Apache-2.0 | 1 |
| CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | 1 |
| CC0-1.0 OR MIT-0 OR Apache-2.0 | 1 |
| ISC | 1 |
| MIT / Apache-2.0 | 1 |
| MIT OR Apache-2.0 OR BSD-1-Clause | 1 |

## Inventário do `Cargo.lock` (621 pacotes)

Crate, versão, licença declarada e origem, ordenados por nome.

| Crate | Versão | Licença | Origem |
|---|---|---|---|
| accesskit | 0.25.2 | MIT OR Apache-2.0 | crates.io |
| accesskit_consumer | 0.39.2 | MIT OR Apache-2.0 | crates.io |
| addr2line | 0.26.1 | Apache-2.0 OR MIT | crates.io |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | crates.io |
| ahash | 0.8.12 | MIT OR Apache-2.0 | crates.io |
| aho-corasick | 1.1.5 | Unlicense OR MIT | crates.io |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 | crates.io |
| android-activity | 0.6.1 | MIT OR Apache-2.0 | crates.io |
| android-properties | 0.2.2 | MIT | crates.io |
| android_system_properties | 0.1.6 | MIT OR Apache-2.0 | crates.io |
| annotate-snippets | 0.12.17 | MIT OR Apache-2.0 | crates.io |
| anstream | 1.0.0 | MIT OR Apache-2.0 | crates.io |
| anstyle | 1.0.14 | MIT OR Apache-2.0 | crates.io |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 | crates.io |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 | crates.io |
| anstyle-wincon | 3.0.11 | MIT OR Apache-2.0 | crates.io |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | crates.io |
| arbitrary | 1.5.0 | MIT OR Apache-2.0 | crates.io |
| arboard | 3.7.0 | MIT OR Apache-2.0 | crates.io |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | crates.io |
| as-raw-xcb-connection | 1.0.1 | MIT OR Apache-2.0 | crates.io |
| ascii_tree | 0.1.1 | MIT | crates.io |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 | crates.io |
| async-trait | 0.1.92 | MIT OR Apache-2.0 | crates.io |
| atoi | 2.0.0 | MIT | crates.io |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | crates.io |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | crates.io |
| base64 | 0.22.1 | MIT OR Apache-2.0 | crates.io |
| base64 | 0.23.1 | MIT OR Apache-2.0 | crates.io |
| bincode | 2.0.1 | MIT | crates.io |
| bincode_derive | 2.0.1 | MIT | crates.io |
| bit-set | 0.10.0 | Apache-2.0 OR MIT | crates.io |
| bit-set | 0.11.1 | Apache-2.0 OR MIT | crates.io |
| bit-vec | 0.10.1 | Apache-2.0 OR MIT | crates.io |
| bit-vec | 0.9.1 | Apache-2.0 OR MIT | crates.io |
| bitflags | 1.3.2 | MIT/Apache-2.0 | crates.io |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | crates.io |
| bitvec | 1.1.1 | MIT | crates.io |
| blake3 | 1.8.7 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | crates.io |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | crates.io |
| block-buffer | 0.12.1 | MIT OR Apache-2.0 | crates.io |
| block2 | 0.5.1 | MIT | crates.io |
| block2 | 0.6.2 | MIT | crates.io |
| borsh | 1.8.1 | MIT OR Apache-2.0 | crates.io |
| borsh-derive | 1.8.1 | Apache-2.0 | crates.io |
| bstr | 1.13.1 | MIT OR Apache-2.0 | crates.io |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 | crates.io |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | crates.io |
| byteorder | 1.5.0 | Unlicense OR MIT | crates.io |
| byteorder-lite | 0.1.0 | Unlicense OR MIT | crates.io |
| bytes | 1.12.1 | MIT | crates.io |
| calloop | 0.13.0 | MIT | crates.io |
| calloop | 0.14.5 | MIT | crates.io |
| calloop-wayland-source | 0.3.0 | MIT | crates.io |
| calloop-wayland-source | 0.4.1 | MIT | crates.io |
| cc | 1.10.0 | MIT OR Apache-2.0 | crates.io |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | crates.io |
| cfg_aliases | 0.2.2 | MIT | crates.io |
| cgl | 0.3.2 | MIT / Apache-2.0 | crates.io |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 | crates.io |
| chrono | 0.4.45 | MIT OR Apache-2.0 | crates.io |
| clap | 4.6.7 | MIT OR Apache-2.0 | crates.io |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 | crates.io |
| clap_derive | 4.6.7 | MIT OR Apache-2.0 | crates.io |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 | crates.io |
| clipboard-win | 5.4.3 | BSL-1.0 | crates.io |
| cmov | 0.5.4 | Apache-2.0 OR MIT | crates.io |
| cobs | 0.3.0 | MIT OR Apache-2.0 | crates.io |
| codespan-reporting | 0.13.1 | Apache-2.0 | crates.io |
| color | 0.3.3 | Apache-2.0 OR MIT | crates.io |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 | crates.io |
| combine | 4.6.9 | MIT | crates.io |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | crates.io |
| constant_time_eq | 0.4.2 | CC0-1.0 OR MIT-0 OR Apache-2.0 | crates.io |
| core-foundation | 0.9.4 | MIT OR Apache-2.0 | crates.io |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 | crates.io |
| core-graphics | 0.23.2 | MIT OR Apache-2.0 | crates.io |
| core-graphics-types | 0.1.3 | MIT OR Apache-2.0 | crates.io |
| core_detect | 1.0.0 | MIT/Apache-2.0 | crates.io |
| countme | 3.0.1 | MIT OR Apache-2.0 | crates.io |
| cpp_demangle | 0.5.1 | MIT OR Apache-2.0 | crates.io |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | crates.io |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | crates.io |
| cranelift-assembler-x64 | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-assembler-x64-meta | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-bforest | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-bitset | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-codegen | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-codegen-meta | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-codegen-shared | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-control | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-entity | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-frontend | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-isle | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-native | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| cranelift-srcgen | 0.135.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| crc | 3.4.0 | MIT OR Apache-2.0 | crates.io |
| crc-catalog | 2.5.0 | MIT OR Apache-2.0 | crates.io |
| crc32fast | 1.5.3 | MIT OR Apache-2.0 | crates.io |
| crossbeam-deque | 0.8.8 | MIT OR Apache-2.0 | crates.io |
| crossbeam-epoch | 0.9.21 | MIT OR Apache-2.0 | crates.io |
| crossbeam-queue | 0.3.14 | MIT OR Apache-2.0 | crates.io |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | crates.io |
| crunchy | 0.2.4 | MIT | crates.io |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | crates.io |
| crypto-common | 0.2.2 | MIT OR Apache-2.0 | crates.io |
| ctutils | 0.4.7 | Apache-2.0 OR MIT | crates.io |
| cursor-icon | 1.2.0 | MIT OR Apache-2.0 OR Zlib | crates.io |
| curve25519-dalek | 5.0.0 | BSD-3-Clause | crates.io |
| curve25519-dalek-derive | 0.1.1 | MIT/Apache-2.0 | crates.io |
| daachorse | 5.1.0 | MIT OR Apache-2.0 | crates.io |
| darling | 0.24.1 | MIT | crates.io |
| darling_core | 0.24.1 | MIT | crates.io |
| darling_macro | 0.24.1 | MIT | crates.io |
| dfir-app | 0.1.0 | Apache-2.0 | workspace (código próprio) |
| dfir-engine | 0.1.0 | Apache-2.0 | workspace (código próprio) |
| dfir-protocol | 0.1.0 | Apache-2.0 | workspace (código próprio) |
| digest | 0.10.7 | MIT OR Apache-2.0 | crates.io |
| digest | 0.11.3 | MIT OR Apache-2.0 | crates.io |
| dispatch | 0.2.0 | MIT | crates.io |
| dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT | crates.io |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 | crates.io |
| dlib | 0.5.3 | MIT | crates.io |
| document-features | 0.2.12 | MIT OR Apache-2.0 | crates.io |
| dotenvy | 0.15.7 | MIT | crates.io |
| downcast-rs | 1.2.1 | MIT/Apache-2.0 | crates.io |
| dpi | 0.1.2 | Apache-2.0 AND MIT | crates.io |
| ecolor | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| ed25519 | 3.0.0 | Apache-2.0 OR MIT | crates.io |
| ed25519-dalek | 3.0.0 | BSD-3-Clause | crates.io |
| eframe | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| egui | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| egui-wgpu | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| egui-winit | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| egui_extras | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| egui_glow | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| egui_kittest | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| either | 1.19.0 | MIT OR Apache-2.0 | crates.io |
| emath | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| embedded-io | 0.4.0 | MIT OR Apache-2.0 | crates.io |
| embedded-io | 0.6.1 | MIT OR Apache-2.0 | crates.io |
| encoding_rs | 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause | crates.io |
| enum-map | 3.1.0 | MIT OR Apache-2.0 | crates.io |
| enum-map-derive | 1.0.1 | MIT OR Apache-2.0 | crates.io |
| epaint | 0.37.0 | MIT OR Apache-2.0 | crates.io |
| epaint_default_fonts | 0.37.0 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 | crates.io |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | crates.io |
| errno | 0.3.14 | MIT OR Apache-2.0 | crates.io |
| error-code | 3.4.0 | BSL-1.0 | crates.io |
| etcetera | 0.11.0 | MIT OR Apache-2.0 | crates.io |
| euclid | 0.22.14 | MIT OR Apache-2.0 | crates.io |
| event-listener | 5.4.2 | Apache-2.0 OR MIT | crates.io |
| evtx | 0.15.1 | MIT/Apache-2.0 | crates.io |
| fallible-iterator | 0.3.0 | MIT/Apache-2.0 | crates.io |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | crates.io |
| faststr | 0.2.34 | MIT OR Apache-2.0 | crates.io |
| fax | 0.2.7 | MIT | crates.io |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | crates.io |
| fearless_simd | 0.4.1 | Apache-2.0 OR MIT | crates.io |
| fiat-crypto | 0.3.0 | MIT OR Apache-2.0 OR BSD-1-Clause | crates.io |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 | crates.io |
| fixedbitset | 0.5.9 | MIT OR Apache-2.0 | crates.io |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | crates.io |
| flume | 0.12.0 | Apache-2.0/MIT | crates.io |
| fnv | 1.0.7 | Apache-2.0 / MIT | crates.io |
| foldhash | 0.1.5 | Zlib | crates.io |
| foldhash | 0.2.0 | Zlib | crates.io |
| font-types | 0.12.7 | MIT OR Apache-2.0 | crates.io |
| foreign-types | 0.5.0 | MIT/Apache-2.0 | crates.io |
| foreign-types-macros | 0.2.4 | MIT/Apache-2.0 | crates.io |
| foreign-types-shared | 0.3.1 | MIT/Apache-2.0 | crates.io |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 | crates.io |
| funty | 2.0.0 | MIT | crates.io |
| futures | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| futures-channel | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| futures-executor | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| futures-intrusive | 0.5.0 | MIT OR Apache-2.0 | crates.io |
| futures-io | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| futures-sink | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| generic-array | 0.14.7 | MIT | crates.io |
| gethostname | 1.1.0 | Apache-2.0 | crates.io |
| getrandom | 0.3.4 | MIT OR Apache-2.0 | crates.io |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | crates.io |
| gimli | 0.32.3 | MIT OR Apache-2.0 | crates.io |
| gimli | 0.33.0 | MIT OR Apache-2.0 | crates.io |
| gl_generator | 0.14.0 | Apache-2.0 | crates.io |
| glifo | 0.3.0 | Apache-2.0 OR MIT | crates.io |
| globset | 0.4.21 | Unlicense OR MIT | crates.io |
| globwalk | 0.9.1 | MIT | crates.io |
| glow | 0.17.0 | MIT OR Apache-2.0 OR Zlib | crates.io |
| glutin | 0.32.3 | Apache-2.0 | crates.io |
| glutin-winit | 0.5.0 | MIT | crates.io |
| glutin_egl_sys | 0.7.1 | Apache-2.0 | crates.io |
| glutin_glx_sys | 0.6.1 | Apache-2.0 | crates.io |
| glutin_wgl_sys | 0.6.1 | Apache-2.0 | crates.io |
| gpu-allocator | 0.28.0 | MIT OR Apache-2.0 | crates.io |
| guillotiere | 0.7.0 | MIT/Apache-2.0 | crates.io |
| half | 2.7.1 | MIT OR Apache-2.0 | crates.io |
| harfrust | 0.12.0 | MIT | crates.io |
| hashbrown | 0.14.5 | MIT OR Apache-2.0 | crates.io |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | crates.io |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 | crates.io |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | crates.io |
| hashlink | 0.11.1 | MIT OR Apache-2.0 | crates.io |
| heck | 0.5.0 | MIT OR Apache-2.0 | crates.io |
| hermit-abi | 0.5.3 | MIT OR Apache-2.0 | crates.io |
| hex | 0.4.3 | MIT OR Apache-2.0 | crates.io |
| hkdf | 0.13.0 | MIT OR Apache-2.0 | crates.io |
| hmac | 0.13.0 | MIT OR Apache-2.0 | crates.io |
| hybrid-array | 0.4.15 | MIT OR Apache-2.0 | crates.io |
| iana-time-zone | 0.1.65 | MIT OR Apache-2.0 | crates.io |
| iana-time-zone-haiku | 0.1.2 | MIT OR Apache-2.0 | crates.io |
| icu_collections | 2.4.0 | Unicode-3.0 | crates.io |
| icu_locale_core | 2.4.0 | Unicode-3.0 | crates.io |
| icu_normalizer | 2.4.0 | Unicode-3.0 | crates.io |
| icu_normalizer_data | 2.4.0 | Unicode-3.0 | crates.io |
| icu_properties | 2.4.0 | Unicode-3.0 | crates.io |
| icu_properties_data | 2.4.0 | Unicode-3.0 | crates.io |
| icu_provider | 2.4.0 | Unicode-3.0 | crates.io |
| id-arena | 2.3.0 | MIT/Apache-2.0 | crates.io |
| ident_case | 1.0.1 | MIT/Apache-2.0 | crates.io |
| idna | 1.1.0 | MIT OR Apache-2.0 | crates.io |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | crates.io |
| ignore | 0.4.34 | Unlicense OR MIT | crates.io |
| image | 0.25.10 | MIT OR Apache-2.0 | crates.io |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | crates.io |
| intaglio | 1.15.1 | Apache-2.0 OR MIT | crates.io |
| inventory | 0.3.25 | MIT OR Apache-2.0 | crates.io |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 | crates.io |
| itertools | 0.14.0 | MIT OR Apache-2.0 | crates.io |
| itertools | 0.15.0 | MIT OR Apache-2.0 | crates.io |
| itoa | 1.0.18 | MIT OR Apache-2.0 | crates.io |
| jiff | 0.2.44 | Unlicense OR MIT | crates.io |
| jiff-core | 0.1.6 | Unlicense OR MIT | crates.io |
| jiff-static | 0.2.44 | Unlicense OR MIT | crates.io |
| jiff-tzdb | 0.1.9 | Unlicense OR MIT | crates.io |
| jiff-tzdb-platform | 0.1.3 | Unlicense OR MIT | crates.io |
| jni | 0.22.4 | MIT OR Apache-2.0 | crates.io |
| jni-macros | 0.22.4 | MIT OR Apache-2.0 | crates.io |
| jni-sys | 0.3.1 | MIT OR Apache-2.0 | crates.io |
| jni-sys | 0.4.1 | MIT OR Apache-2.0 | crates.io |
| jni-sys-macros | 0.4.1 | MIT OR Apache-2.0 | crates.io |
| jobserver | 0.1.35 | MIT OR Apache-2.0 | crates.io |
| js-sys | 0.3.106 | MIT OR Apache-2.0 | crates.io |
| khronos-egl | 6.0.0 | MIT/Apache-2.0 | crates.io |
| khronos_api | 3.1.0 | Apache-2.0 | crates.io |
| kittest | 0.5.0 | MIT OR Apache-2.0 | crates.io |
| kurbo | 0.13.1 | Apache-2.0 OR MIT | crates.io |
| lazy_static | 1.5.1 | MIT OR Apache-2.0 | crates.io |
| leb128 | 0.2.7 | MIT OR Apache-2.0 | crates.io |
| leb128fmt | 0.1.0 | MIT OR Apache-2.0 | crates.io |
| libc | 0.2.191 | MIT OR Apache-2.0 | crates.io |
| libloading | 0.8.9 | ISC | crates.io |
| libm | 0.2.16 | MIT | crates.io |
| libredox | 0.1.25 | MIT | crates.io |
| libsqlite3-sys | 0.40.0 | MIT | crates.io |
| linebender_resource_handle | 0.1.1 | Apache-2.0 OR MIT | crates.io |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| linux-raw-sys | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| litemap | 0.8.3 | Unicode-3.0 | crates.io |
| litrs | 1.0.1 | MIT OR Apache-2.0 | crates.io |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | crates.io |
| log | 0.4.34 | MIT OR Apache-2.0 | crates.io |
| logos | 0.16.1 | MIT OR Apache-2.0 | crates.io |
| logos-codegen | 0.16.1 | MIT OR Apache-2.0 | crates.io |
| logos-derive | 0.16.1 | MIT OR Apache-2.0 | crates.io |
| mach2 | 0.6.0 | BSD-2-Clause OR MIT OR Apache-2.0 | crates.io |
| matchers | 0.2.0 | MIT | crates.io |
| md-5 | 0.11.0 | MIT OR Apache-2.0 | crates.io |
| memchr | 2.8.3 | Unlicense OR MIT | crates.io |
| memfd | 0.6.6 | MIT OR Apache-2.0 | crates.io |
| memmap2 | 0.9.12 | MIT OR Apache-2.0 | crates.io |
| memoffset | 0.9.1 | MIT | crates.io |
| mime | 0.3.17 | MIT OR Apache-2.0 | crates.io |
| mime_guess2 | 2.3.1 | MIT | crates.io |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | crates.io |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | crates.io |
| mio | 1.2.4 | MIT | crates.io |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 | crates.io |
| multiversion_no_op | 1.0.0 | Apache-2.0 OR MIT | crates.io |
| munge | 0.4.7 | MIT | crates.io |
| munge_macro | 0.4.7 | MIT | crates.io |
| naga | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| naga-types | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| ndk | 0.9.0 | MIT OR Apache-2.0 | crates.io |
| ndk-context | 0.1.1 | MIT OR Apache-2.0 | crates.io |
| ndk-sys | 0.6.0+11769913 | MIT OR Apache-2.0 | crates.io |
| nohash-hasher | 0.2.0 | Apache-2.0 OR MIT | crates.io |
| nom | 8.0.0 | MIT | crates.io |
| nu-ansi-term | 0.50.3 | MIT | crates.io |
| num-derive | 0.3.3 | MIT OR Apache-2.0 | crates.io |
| num-derive | 0.5.1 | MIT OR Apache-2.0 | crates.io |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | crates.io |
| num_enum | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | crates.io |
| num_enum_derive | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | crates.io |
| objc-sys | 0.3.5 | MIT | crates.io |
| objc2 | 0.5.3 | MIT | crates.io |
| objc2 | 0.6.5 | MIT | crates.io |
| objc2-app-kit | 0.2.2 | MIT | crates.io |
| objc2-app-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-cloud-kit | 0.2.2 | MIT | crates.io |
| objc2-contacts | 0.2.2 | MIT | crates.io |
| objc2-core-data | 0.2.2 | MIT | crates.io |
| objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-core-graphics | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-core-image | 0.2.2 | MIT | crates.io |
| objc2-core-location | 0.2.2 | MIT | crates.io |
| objc2-encode | 4.1.0 | MIT | crates.io |
| objc2-foundation | 0.2.2 | MIT | crates.io |
| objc2-foundation | 0.3.2 | MIT | crates.io |
| objc2-io-surface | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-link-presentation | 0.2.2 | MIT | crates.io |
| objc2-metal | 0.2.2 | MIT | crates.io |
| objc2-metal | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-quartz-core | 0.2.2 | MIT | crates.io |
| objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-symbols | 0.2.2 | MIT | crates.io |
| objc2-system-configuration | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-ui-kit | 0.2.2 | MIT | crates.io |
| objc2-ui-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-uniform-type-identifiers | 0.2.2 | MIT | crates.io |
| objc2-uniform-type-identifiers | 0.3.2 | Zlib OR Apache-2.0 OR MIT | crates.io |
| objc2-user-notifications | 0.2.2 | MIT | crates.io |
| object | 0.39.1 | Apache-2.0 OR MIT | crates.io |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | crates.io |
| once_cell_polyfill | 1.70.2 | MIT OR Apache-2.0 | crates.io |
| orbclient | 0.3.55 | MIT | crates.io |
| ordered-float | 5.5.1 | MIT | crates.io |
| os_pipe | 1.2.3 | MIT | crates.io |
| parking | 2.2.1 | Apache-2.0 OR MIT | crates.io |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | crates.io |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | crates.io |
| peniko | 0.6.1 | Apache-2.0 OR MIT | crates.io |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 | crates.io |
| petgraph | 0.8.3 | MIT OR Apache-2.0 | crates.io |
| phf | 0.11.3 | MIT | crates.io |
| phf_generator | 0.11.3 | MIT | crates.io |
| phf_macros | 0.11.3 | MIT | crates.io |
| phf_shared | 0.11.3 | MIT | crates.io |
| pin-project | 1.1.13 | Apache-2.0 OR MIT | crates.io |
| pin-project-internal | 1.1.13 | Apache-2.0 OR MIT | crates.io |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | crates.io |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | crates.io |
| plain | 0.2.3 | MIT/Apache-2.0 | crates.io |
| png | 0.18.1 | MIT OR Apache-2.0 | crates.io |
| polling | 3.11.0 | Apache-2.0 OR MIT | crates.io |
| pollster | 1.0.1 | Apache-2.0/MIT | crates.io |
| polycool | 0.4.0 | MIT OR Apache-2.0 | crates.io |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT | crates.io |
| portable-atomic-util | 0.2.8 | Apache-2.0 OR MIT | crates.io |
| postcard | 1.1.3 | MIT OR Apache-2.0 | crates.io |
| potential_utf | 0.1.6 | Unicode-3.0 | crates.io |
| presser | 0.3.1 | MIT OR Apache-2.0 | crates.io |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 | crates.io |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | crates.io |
| profiling | 1.0.18 | MIT OR Apache-2.0 | crates.io |
| proptest | 1.12.0 | MIT OR Apache-2.0 | crates.io |
| protobuf | 3.7.2 | MIT | crates.io |
| protobuf-support | 3.7.2 | MIT | crates.io |
| ptr_meta | 0.3.2 | MIT | crates.io |
| ptr_meta_derive | 0.3.2 | MIT | crates.io |
| pulley-interpreter | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| pulley-macros | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 | crates.io |
| quick-error | 1.2.3 | MIT/Apache-2.0 | crates.io |
| quick-error | 2.0.1 | MIT/Apache-2.0 | crates.io |
| quick-xml | 0.41.0 | MIT | crates.io |
| quote | 1.0.47 | MIT OR Apache-2.0 | crates.io |
| r-efi | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | crates.io |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | crates.io |
| radium | 0.7.0 | MIT | crates.io |
| rancor | 0.1.3 | MIT | crates.io |
| rand | 0.10.3 | MIT OR Apache-2.0 | crates.io |
| rand | 0.8.8 | MIT OR Apache-2.0 | crates.io |
| rand_core | 0.10.1 | MIT OR Apache-2.0 | crates.io |
| rand_core | 0.6.4 | MIT OR Apache-2.0 | crates.io |
| rand_xorshift | 0.5.0 | MIT OR Apache-2.0 | crates.io |
| range-alloc | 0.1.5 | MIT OR Apache-2.0 | crates.io |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | crates.io |
| raw-window-metal | 1.1.0 | MIT OR Apache-2.0 | crates.io |
| read-fonts | 0.41.0 | MIT OR Apache-2.0 | crates.io |
| redox_syscall | 0.4.1 | MIT | crates.io |
| redox_syscall | 0.5.18 | MIT | crates.io |
| redox_syscall | 0.9.5 | MIT | crates.io |
| ref-cast | 1.0.27 | MIT OR Apache-2.0 | crates.io |
| ref-cast-impl | 1.0.27 | MIT OR Apache-2.0 | crates.io |
| regalloc2 | 0.15.2 | Apache-2.0 WITH LLVM-exception | crates.io |
| regex | 1.13.1 | MIT OR Apache-2.0 | crates.io |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | crates.io |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | crates.io |
| rend | 0.5.4 | MIT | crates.io |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 | crates.io |
| rfd | 0.18.0 | MIT | crates.io |
| rkyv | 0.8.18 | MIT | crates.io |
| rkyv_derive | 0.8.18 | MIT | crates.io |
| rowan | 0.17.0 | MIT OR Apache-2.0 | crates.io |
| rustc-demangle | 0.1.28 | MIT/Apache-2.0 | crates.io |
| rustc-hash | 1.1.0 | Apache-2.0/MIT | crates.io |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | crates.io |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 | crates.io |
| rustix | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| rustix | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | crates.io |
| rusty-fork | 0.3.1 | MIT/Apache-2.0 | crates.io |
| ryu-js | 1.0.3 | Apache-2.0 OR BSL-1.0 | crates.io |
| same-file | 1.0.6 | Unlicense/MIT | crates.io |
| scoped-tls | 1.0.1 | MIT/Apache-2.0 | crates.io |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | crates.io |
| self_cell | 1.3.0 | Apache-2.0 OR GPL-2.0-only | crates.io |
| semver | 1.0.28 | MIT OR Apache-2.0 | crates.io |
| send_wrapper | 0.6.0 | MIT/Apache-2.0 | crates.io |
| serde | 1.0.229 | MIT OR Apache-2.0 | crates.io |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | crates.io |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | crates.io |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | crates.io |
| serde_json_canonicalizer | 0.3.2 | MIT | crates.io |
| serde_spanned | 1.1.2 | MIT OR Apache-2.0 | crates.io |
| sha1 | 0.11.0 | MIT OR Apache-2.0 | crates.io |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | crates.io |
| sha2 | 0.11.1 | MIT OR Apache-2.0 | crates.io |
| sharded-slab | 0.1.7 | MIT | crates.io |
| shlex | 2.0.1 | MIT OR Apache-2.0 | crates.io |
| signal-hook-registry | 1.4.8 | MIT OR Apache-2.0 | crates.io |
| signature | 3.0.0 | Apache-2.0 OR MIT | crates.io |
| simd-adler32 | 0.3.10 | MIT | crates.io |
| simd_cesu8 | 1.2.0 | Apache-2.0 OR MIT | crates.io |
| simdutf8 | 0.1.5 | MIT OR Apache-2.0 | crates.io |
| siphasher | 1.0.4 | MIT OR Apache-2.0 | crates.io |
| skrifa | 0.44.0 | MIT OR Apache-2.0 | crates.io |
| slab | 0.4.12 | MIT | crates.io |
| slotmap | 1.1.1 | Zlib | crates.io |
| smallvec | 1.16.4 | MIT OR Apache-2.0 | crates.io |
| smithay-client-toolkit | 0.19.2 | MIT | crates.io |
| smithay-client-toolkit | 0.20.0 | MIT | crates.io |
| smithay-clipboard | 0.7.3 | MIT | crates.io |
| smol_str | 0.2.2 | MIT OR Apache-2.0 | crates.io |
| socket2 | 0.6.5 | MIT OR Apache-2.0 | crates.io |
| sonic-number | 0.1.3 | Apache-2.0 | crates.io |
| sonic-rs | 0.5.10 | Apache-2.0 | crates.io |
| sonic-simd | 0.1.4 | Apache-2.0 | crates.io |
| spin | 0.9.9 | MIT | crates.io |
| spirv | 0.4.0+sdk-1.4.341.0 | Apache-2.0 | crates.io |
| sqlx | 0.9.1 | MIT OR Apache-2.0 | crates.io |
| sqlx-core | 0.9.1 | MIT OR Apache-2.0 | crates.io |
| sqlx-macros | 0.9.1 | MIT OR Apache-2.0 | crates.io |
| sqlx-macros-core | 0.9.1 | MIT OR Apache-2.0 | crates.io |
| sqlx-mysql | 0.9.1 | MIT OR Apache-2.0 | crates.io |
| sqlx-postgres | 0.9.1 | MIT OR Apache-2.0 | crates.io |
| sqlx-sqlite | 0.9.1 | MIT OR Apache-2.0 | crates.io |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | crates.io |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | crates.io |
| stringprep | 0.1.5 | MIT/Apache-2.0 | crates.io |
| strsim | 0.11.1 | MIT | crates.io |
| strum_macros | 0.28.0 | MIT | crates.io |
| subtle | 2.6.1 | BSD-3-Clause | crates.io |
| syn | 1.0.109 | MIT OR Apache-2.0 | crates.io |
| syn | 2.0.119 | MIT OR Apache-2.0 | crates.io |
| syn | 3.0.9 | MIT OR Apache-2.0 | crates.io |
| synstructure | 0.14.0 | MIT | crates.io |
| tap | 1.0.1 | MIT | crates.io |
| target-lexicon | 0.13.5 | Apache-2.0 WITH LLVM-exception | crates.io |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | crates.io |
| termcolor | 1.4.1 | Unlicense OR MIT | crates.io |
| text-size | 1.1.1 | MIT OR Apache-2.0 | crates.io |
| thiserror | 1.0.69 | MIT OR Apache-2.0 | crates.io |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | crates.io |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | crates.io |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | crates.io |
| thread_local | 1.1.10 | MIT OR Apache-2.0 | crates.io |
| tiff | 0.11.3 | MIT | crates.io |
| tinystr | 0.8.5 | Unicode-3.0 | crates.io |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT | crates.io |
| tokio | 1.54.1 | MIT | crates.io |
| tokio-macros | 2.7.2 | MIT | crates.io |
| tokio-stream | 0.1.19 | MIT | crates.io |
| tokio-util | 0.7.20 | MIT | crates.io |
| toml | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 | crates.io |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | crates.io |
| toml_edit | 0.25.17+spec-1.1.0 | MIT OR Apache-2.0 | crates.io |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | crates.io |
| tracing | 0.1.44 | MIT | crates.io |
| tracing-attributes | 0.1.31 | MIT | crates.io |
| tracing-core | 0.1.36 | MIT | crates.io |
| tracing-log | 0.2.0 | MIT | crates.io |
| tracing-serde | 0.2.0 | MIT | crates.io |
| tracing-subscriber | 0.3.23 | MIT | crates.io |
| tree_magic_mini | 3.2.2 | MIT | crates.io |
| type-map | 0.5.1 | MIT/Apache-2.0 | crates.io |
| typenum | 1.20.1 | MIT OR Apache-2.0 | crates.io |
| unarray | 0.1.4 | MIT OR Apache-2.0 | crates.io |
| unicase | 2.10.0 | MIT OR Apache-2.0 | crates.io |
| unicode-bidi | 0.3.18 | MIT OR Apache-2.0 | crates.io |
| unicode-general-category | 1.1.0 | Apache-2.0 | crates.io |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | crates.io |
| unicode-normalization | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | crates.io |
| unicode-properties | 0.1.4 | MIT/Apache-2.0 | crates.io |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | crates.io |
| unicode-width | 0.2.3 | MIT OR Apache-2.0 | crates.io |
| unty | 0.0.4 | MIT OR Apache-2.0 | crates.io |
| url | 2.5.8 | MIT OR Apache-2.0 | crates.io |
| utf16-simd | 0.2.0 | MIT OR Apache-2.0 | crates.io |
| utf8_iter | 1.1.0 | Apache-2.0 OR MIT | crates.io |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | crates.io |
| utf_types | 1.0.0 | Apache-2.0 OR MIT | crates.io |
| uuid | 1.29.2 | Apache-2.0 OR MIT | crates.io |
| valuable | 0.1.1 | MIT | crates.io |
| vcpkg | 0.2.15 | MIT/Apache-2.0 | crates.io |
| vello_common | 0.2.0 | Apache-2.0 OR MIT | crates.io |
| vello_cpu | 0.2.0 | Apache-2.0 OR MIT | crates.io |
| version_check | 0.9.5 | MIT/Apache-2.0 | crates.io |
| virtue | 0.0.18 | MIT | crates.io |
| wait-timeout | 0.2.1 | MIT/Apache-2.0 | crates.io |
| walkdir | 2.5.0 | Unlicense/MIT | crates.io |
| walrus | 0.27.3 | MIT/Apache-2.0 | crates.io |
| walrus-macro | 0.26.0 | MIT/Apache-2.0 | crates.io |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| wasm-bindgen | 0.2.129 | MIT OR Apache-2.0 | crates.io |
| wasm-bindgen-futures | 0.4.79 | MIT OR Apache-2.0 | crates.io |
| wasm-bindgen-macro | 0.2.129 | MIT OR Apache-2.0 | crates.io |
| wasm-bindgen-macro-support | 0.2.129 | MIT OR Apache-2.0 | crates.io |
| wasm-bindgen-shared | 0.2.129 | MIT OR Apache-2.0 | crates.io |
| wasm-encoder | 0.245.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| wasm-encoder | 0.254.2 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| wasmparser | 0.245.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| wasmparser | 0.254.2 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| wasmprinter | 0.254.2 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| wasmtime | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-environ | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-internal-component-util | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-internal-core | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-internal-cranelift | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-internal-fiber | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-internal-jit-debug | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-internal-jit-icache-coherence | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-internal-unwinder | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wasmtime-internal-versioned-export-macros | 48.0.9 | Apache-2.0 WITH LLVM-exception | crates.io |
| wayland-backend | 0.3.17 | MIT | crates.io |
| wayland-client | 0.31.15 | MIT | crates.io |
| wayland-csd-frame | 0.3.0 | MIT | crates.io |
| wayland-cursor | 0.31.14 | MIT | crates.io |
| wayland-protocols | 0.32.13 | MIT | crates.io |
| wayland-protocols-experimental | 20250721.0.1 | MIT | crates.io |
| wayland-protocols-misc | 0.3.12 | MIT | crates.io |
| wayland-protocols-plasma | 0.3.12 | MIT | crates.io |
| wayland-protocols-wlr | 0.3.12 | MIT | crates.io |
| wayland-scanner | 0.31.11 | MIT | crates.io |
| wayland-sys | 0.31.11 | MIT | crates.io |
| web-sys | 0.3.106 | MIT OR Apache-2.0 | crates.io |
| web-time | 1.1.0 | MIT OR Apache-2.0 | crates.io |
| webbrowser | 1.2.5 | MIT OR Apache-2.0 | crates.io |
| weezl | 0.1.12 | MIT OR Apache-2.0 | crates.io |
| wgpu | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| wgpu-core | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| wgpu-core-deps-apple | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| wgpu-core-deps-emscripten | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| wgpu-core-deps-wasm | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| wgpu-core-deps-windows-linux-android | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| wgpu-hal | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| wgpu-naga-bridge | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| wgpu-types | 30.0.1 | MIT OR Apache-2.0 | crates.io |
| whoami | 2.1.3 | Apache-2.0 OR BSL-1.0 OR MIT | crates.io |
| winapi-util | 0.1.11 | Unlicense OR MIT | crates.io |
| windows | 0.62.2 | MIT OR Apache-2.0 | crates.io |
| windows-collections | 0.3.2 | MIT OR Apache-2.0 | crates.io |
| windows-core | 0.62.2 | MIT OR Apache-2.0 | crates.io |
| windows-future | 0.3.2 | MIT OR Apache-2.0 | crates.io |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 | crates.io |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 | crates.io |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | crates.io |
| windows-numerics | 0.3.1 | MIT OR Apache-2.0 | crates.io |
| windows-result | 0.4.1 | MIT OR Apache-2.0 | crates.io |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 | crates.io |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 | crates.io |
| windows-sys | 0.59.0 | MIT OR Apache-2.0 | crates.io |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | crates.io |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| windows-threading | 0.2.1 | MIT OR Apache-2.0 | crates.io |
| windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 | crates.io |
| winit | 0.30.13 | Apache-2.0 | crates.io |
| winnow | 1.0.4 | MIT | crates.io |
| winstructs | 0.3.2 | MIT/Apache-2.0 | crates.io |
| wit-bindgen | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | crates.io |
| wl-clipboard-rs | 0.9.5 | MIT OR Apache-2.0 | crates.io |
| writeable | 0.6.4 | Unicode-3.0 | crates.io |
| wyz | 0.5.1 | MIT | crates.io |
| x11-dl | 2.21.0 | MIT | crates.io |
| x11rb | 0.13.2 | MIT OR Apache-2.0 | crates.io |
| x11rb-protocol | 0.13.2 | MIT OR Apache-2.0 | crates.io |
| xcursor | 0.3.11 | MIT | crates.io |
| xkbcommon-dl | 0.4.2 | MIT | crates.io |
| xkeysym | 0.2.1 | MIT OR Apache-2.0 OR Zlib | crates.io |
| xml-rs | 0.8.29 | MIT | crates.io |
| yara-x | 1.22.1 | BSD-3-Clause | crates.io |
| yara-x-macros | 1.22.1 | BSD-3-Clause | crates.io |
| yara-x-parser | 1.22.1 | BSD-3-Clause | crates.io |
| yoke | 0.8.4 | Unicode-3.0 | crates.io |
| yoke-derive | 0.8.4 | Unicode-3.0 | crates.io |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | crates.io |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | crates.io |
| zerofrom | 0.1.8 | Unicode-3.0 | crates.io |
| zerofrom-derive | 0.1.8 | Unicode-3.0 | crates.io |
| zeroize | 1.9.2 | Apache-2.0 OR MIT | crates.io |
| zerotrie | 0.2.5 | Unicode-3.0 | crates.io |
| zerovec | 0.11.9 | Unicode-3.0 | crates.io |
| zerovec-derive | 0.11.7 | Unicode-3.0 | crates.io |
| zlib-rs | 0.6.8 | Zlib | crates.io |
| zmij | 1.0.23 | MIT | crates.io |
| zune-core | 0.5.3 | MIT OR Apache-2.0 OR Zlib | crates.io |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib | crates.io |
