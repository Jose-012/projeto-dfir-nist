//! Contenção do processo do engine pelo sistema operacional (F1-22, RF-033,
//! ADR-0024).
//!
//! O engine lê o log, que é **entrada hostil**, com `evtx` e YARA-X. Se um
//! desses parsers tiver uma falha explorável, o atacante passa a executar
//! código com os privilégios do processo. Este módulo reduz o que esse código
//! consegue fazer, **antes** de o engine ler qualquer byte da evidência:
//!
//! - `no_new_privs`: o processo nunca ganha privilégio por `exec`;
//! - **Landlock** (sistema de arquivos): o engine só lê o arquivo de evidência
//!   daquele job (e, no smoke de processo, escreve na pasta de saída do job).
//!   Qualquer outro `open` falha com `EACCES`. Com o kernel recente também
//!   nega bind/connect TCP e isola sockets Unix abstratos e sinais;
//! - **seccomp** (chamadas de sistema): nega criar socket (rede e Unix),
//!   `execve`, `ptrace`, `mount`, `bpf`, `io_uring`, carregar módulo e afins;
//! - `RLIMIT_CORE = 0`: nenhum core dump com trechos da evidência.
//!
//! O que isto **não** faz: não impede um laço infinito nem esgotar memória
//! (isso é do supervisor: `RLIMIT_AS`, timeout e encerramento no app); não
//! cobre Windows (a contenção ali é processo separado, ambiente reduzido e
//! arquivo único; ver ADR-0024); não bloqueia `clone` com flags de namespace,
//! porque o filtro clássico do seccomp não enxerga os argumentos de `clone3`.
//!
//! **Falha fechada** (RF-036): com a política `required`, se uma camada não
//! puder ser aplicada, o engine **não analisa**: responde `SANDBOX_UNAVAILABLE`
//! a cada pedido. Não existe queda silenciosa para "sem sandbox". A política
//! `best-effort` e a `off` existem, são escolha explícita do operador
//! (`DFIR_ENGINE_SANDBOX`) e o estado real vai para o job como aviso.

use std::path::PathBuf;

/// Variável de ambiente que escolhe a política.
pub const POLICY_ENV: &str = "DFIR_ENGINE_SANDBOX";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Toda camada disponível para o sistema precisa ser aplicada.
    Required,
    /// Aplica o que o sistema permite e informa o que faltou.
    BestEffort,
    /// Não aplica nada (só para depuração; fica registrado no job).
    Off,
}

impl Policy {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "required" => Some(Self::Required),
            "best-effort" | "best_effort" | "besteffort" => Some(Self::BestEffort),
            "off" | "none" | "disabled" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::BestEffort => "best-effort",
            Self::Off => "off",
        }
    }

    /// Padrão por sistema: `required` onde há contenção implementada (Linux),
    /// `best-effort` onde ela ainda não existe (o estado vai para o job).
    pub fn platform_default() -> Self {
        if cfg!(target_os = "linux") {
            Self::Required
        } else {
            Self::BestEffort
        }
    }

    /// Lê `DFIR_ENGINE_SANDBOX`. Valor inválido é **falha fechada**: vira
    /// `Required` (um erro de digitação não desliga a contenção).
    pub fn from_env() -> Self {
        match std::env::var(POLICY_ENV) {
            Ok(raw) => Self::parse(&raw).unwrap_or(Self::Required),
            Err(_) => Self::platform_default(),
        }
    }
}

/// O que o processo precisa acessar. Tudo o mais é negado.
#[derive(Debug, Clone, Default)]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub struct Plan {
    /// Único arquivo de evidência que pode ser lido (modo de produção).
    pub read_file: Option<PathBuf>,
    /// Pasta lida por inteiro (modo manual/QA, sem arquivo fixado).
    pub read_dir: Option<PathBuf>,
    /// Pasta onde o engine pode criar e escrever arquivos (smoke de processo).
    pub write_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layer {
    /// Aplicada, com um detalhe curto (`abi7`, `on`).
    Applied(String),
    /// Não pôde ser aplicada neste sistema, com o motivo.
    Unavailable(String),
    /// Política `off`.
    Skipped,
    /// A camada não existe neste sistema operacional (Job Object no Linux,
    /// por exemplo). Não conta como falha nem como contenção ausente.
    NotApplicable,
}

impl Layer {
    #[cfg(target_os = "linux")]
    fn is_applied(&self) -> bool {
        matches!(self, Layer::Applied(_))
    }

    fn token(&self) -> String {
        match self {
            Layer::Applied(detail) => detail.clone(),
            Layer::Unavailable(_) => "unavailable".into(),
            Layer::Skipped => "skipped".into(),
            Layer::NotApplicable => "na".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub policy: Policy,
    pub no_new_privs: Layer,
    pub landlock: Layer,
    pub seccomp: Layer,
    pub core_dumps: Layer,
    /// Job Object do Windows: limite de memória, nenhum processo filho e sem
    /// acesso à interface do usuário.
    pub job: Layer,
}

impl Status {
    /// Texto curto e sem caminhos, enviado ao app num `PROGRESS`.
    pub fn summary(&self) -> String {
        format!(
            "policy={};nnp={};landlock={};seccomp={};core={};job={}",
            self.policy.as_str(),
            self.no_new_privs.token(),
            self.landlock.token(),
            self.seccomp.token(),
            self.core_dumps.token(),
            self.job.token(),
        )
    }

    /// Motivos das camadas que não puderam ser aplicadas.
    pub fn missing(&self) -> Vec<String> {
        let mut missing = Vec::new();
        for (name, layer) in [
            ("no_new_privs", &self.no_new_privs),
            ("landlock", &self.landlock),
            ("seccomp", &self.seccomp),
            ("core_dumps", &self.core_dumps),
            ("job", &self.job),
        ] {
            if let Layer::Unavailable(reason) = layer {
                missing.push(format!("{name}: {reason}"));
            }
        }
        missing
    }

    /// Toda camada que existe neste sistema foi aplicada.
    pub fn fully_applied(&self) -> bool {
        [
            &self.no_new_privs,
            &self.landlock,
            &self.seccomp,
            &self.core_dumps,
            &self.job,
        ]
        .into_iter()
        .all(|layer| matches!(layer, Layer::Applied(_) | Layer::NotApplicable))
    }
}

/// Aplica a política. Deve ser chamada **antes** de criar threads e de ler a
/// evidência: Landlock e seccomp valem para a thread que chama e para as que
/// ela criar depois.
///
/// `Err` só ocorre com `Required` e camada indisponível; a mensagem lista o
/// que faltou.
pub fn apply(policy: Policy, plan: &Plan) -> Result<Status, String> {
    if policy == Policy::Off {
        return Ok(Status {
            policy,
            no_new_privs: Layer::Skipped,
            landlock: Layer::Skipped,
            seccomp: Layer::Skipped,
            core_dumps: Layer::Skipped,
            job: Layer::Skipped,
        });
    }
    let status = platform::apply(policy, plan);
    if policy == Policy::Required && !status.fully_applied() {
        return Err(format!(
            "the engine sandbox is required but could not be fully applied ({}); \
             fix the system or choose {POLICY_ENV}=best-effort explicitly",
            status.missing().join("; ")
        ));
    }
    Ok(status)
}

#[cfg(target_os = "linux")]
mod platform {
    use super::{Layer, Plan, Policy, Status};
    use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path};

    // Chamadas de sistema do Landlock: mesmos números em x86_64 e aarch64.
    const SYS_LANDLOCK_CREATE_RULESET: libc::c_long = 444;
    const SYS_LANDLOCK_ADD_RULE: libc::c_long = 445;
    const SYS_LANDLOCK_RESTRICT_SELF: libc::c_long = 446;
    const LANDLOCK_CREATE_RULESET_VERSION: libc::c_ulong = 1;
    const LANDLOCK_RULE_PATH_BENEATH: libc::c_uint = 1;

    // Direitos de sistema de arquivos (ABI 1) e acréscimos de ABIs novas.
    const FS_EXECUTE: u64 = 1 << 0;
    const FS_WRITE_FILE: u64 = 1 << 1;
    const FS_READ_FILE: u64 = 1 << 2;
    const FS_READ_DIR: u64 = 1 << 3;
    const FS_ABI1_ALL: u64 = (1 << 13) - 1;
    const FS_REFER: u64 = 1 << 13; // ABI 2
    const FS_TRUNCATE: u64 = 1 << 14; // ABI 3
    const FS_IOCTL_DEV: u64 = 1 << 15; // ABI 5
    const FS_MAKE_REG: u64 = 1 << 8;
    const NET_BIND_TCP: u64 = 1 << 0; // ABI 4
    const NET_CONNECT_TCP: u64 = 1 << 1;
    const SCOPE_ABSTRACT_UNIX_SOCKET: u64 = 1 << 0; // ABI 6
    const SCOPE_SIGNAL: u64 = 1 << 1;

    #[repr(C)]
    struct RulesetAttr {
        handled_access_fs: u64,
        handled_access_net: u64,
        scoped: u64,
    }

    #[repr(C, packed)]
    struct PathBeneathAttr {
        allowed_access: u64,
        parent_fd: i32,
    }

    fn last_error() -> String {
        std::io::Error::last_os_error().to_string()
    }

    /// Versão da ABI do Landlock, ou o motivo de não haver.
    fn landlock_abi() -> Result<u32, String> {
        // SAFETY: chamada de sistema sem ponteiros (attr nulo, tamanho 0).
        let abi = unsafe {
            libc::syscall(
                SYS_LANDLOCK_CREATE_RULESET,
                std::ptr::null::<RulesetAttr>(),
                0 as libc::size_t,
                LANDLOCK_CREATE_RULESET_VERSION,
            )
        };
        if abi < 0 {
            return Err(format!(
                "Landlock is not available in this kernel ({})",
                last_error()
            ));
        }
        Ok(abi as u32)
    }

    /// Regra que libera `access` abaixo de `path`.
    fn add_path_rule(ruleset_fd: i32, path: &Path, access: u64) -> Result<(), String> {
        let c_path = CString::new(path.as_os_str().as_bytes())
            .map_err(|_| "a sandbox path contains a NUL byte".to_string())?;
        // SAFETY: `c_path` é uma string C válida durante a chamada.
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
        if fd < 0 {
            return Err(format!("cannot open a sandbox path ({})", last_error()));
        }
        let attr = PathBeneathAttr {
            allowed_access: access,
            parent_fd: fd,
        };
        // SAFETY: `attr` vive até o fim da chamada e tem o layout do kernel.
        let rc = unsafe {
            libc::syscall(
                SYS_LANDLOCK_ADD_RULE,
                ruleset_fd,
                LANDLOCK_RULE_PATH_BENEATH,
                &attr as *const PathBeneathAttr,
                0 as libc::c_uint,
            )
        };
        let error = last_error();
        // SAFETY: `fd` foi aberto acima e não é usado depois.
        unsafe { libc::close(fd) };
        if rc != 0 {
            return Err(format!("cannot add a Landlock rule ({error})"));
        }
        Ok(())
    }

    fn apply_landlock(plan: &Plan) -> Layer {
        let abi = match landlock_abi() {
            Ok(abi) if abi >= 1 => abi,
            Ok(_) => return Layer::Unavailable("Landlock reports ABI 0".into()),
            Err(reason) => return Layer::Unavailable(reason),
        };

        let mut handled_fs = FS_ABI1_ALL;
        if abi >= 2 {
            handled_fs |= FS_REFER;
        }
        if abi >= 3 {
            handled_fs |= FS_TRUNCATE;
        }
        if abi >= 5 {
            handled_fs |= FS_IOCTL_DEV;
        }
        let handled_net = if abi >= 4 {
            NET_BIND_TCP | NET_CONNECT_TCP
        } else {
            0
        };
        let scoped = if abi >= 6 {
            SCOPE_ABSTRACT_UNIX_SOCKET | SCOPE_SIGNAL
        } else {
            0
        };
        let attr = RulesetAttr {
            handled_access_fs: handled_fs,
            handled_access_net: handled_net,
            scoped,
        };
        // O tamanho enviado é o que o kernel conhece (campos extras zerados
        // seriam aceitos, mas o tamanho exato é o caminho mais previsível).
        let size = if abi >= 6 {
            std::mem::size_of::<RulesetAttr>()
        } else if abi >= 4 {
            16
        } else {
            8
        };
        // SAFETY: `attr` vive durante a chamada; `size` não passa do struct.
        let ruleset_fd = unsafe {
            libc::syscall(
                SYS_LANDLOCK_CREATE_RULESET,
                &attr as *const RulesetAttr,
                size as libc::size_t,
                0 as libc::c_ulong,
            )
        };
        if ruleset_fd < 0 {
            return Layer::Unavailable(format!(
                "cannot create a Landlock ruleset ({})",
                last_error()
            ));
        }
        let ruleset_fd = ruleset_fd as i32;

        let mut rules: Vec<(&Path, u64)> = Vec::new();
        if let Some(file) = plan.read_file.as_deref() {
            rules.push((file, FS_READ_FILE));
        }
        if let Some(dir) = plan.read_dir.as_deref() {
            rules.push((dir, FS_READ_FILE | FS_READ_DIR));
        }
        if let Some(dir) = plan.write_dir.as_deref() {
            let mut access = FS_WRITE_FILE | FS_MAKE_REG | FS_READ_FILE | FS_READ_DIR;
            if abi >= 3 {
                access |= FS_TRUNCATE;
            }
            rules.push((dir, access));
        }
        // `EXECUTE` fica em `handled` e sem regra: nenhum `exec` por caminho.
        let _ = FS_EXECUTE;

        let mut outcome = Layer::Applied(format!("abi{abi}"));
        for (path, access) in rules {
            if let Err(reason) = add_path_rule(ruleset_fd, path, access) {
                outcome = Layer::Unavailable(reason);
                break;
            }
        }
        if let Layer::Applied(_) = outcome {
            // `no_new_privs` precisa estar ligado antes de restringir.
            // SAFETY: prctl sem ponteiros.
            let nnp = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
            if nnp != 0 {
                outcome =
                    Layer::Unavailable(format!("PR_SET_NO_NEW_PRIVS failed ({})", last_error()));
            } else {
                // SAFETY: `ruleset_fd` é válido; flags 0.
                let rc = unsafe {
                    libc::syscall(SYS_LANDLOCK_RESTRICT_SELF, ruleset_fd, 0 as libc::c_uint)
                };
                if rc != 0 {
                    outcome = Layer::Unavailable(format!(
                        "landlock_restrict_self failed ({})",
                        last_error()
                    ));
                }
            }
        }
        // SAFETY: `ruleset_fd` foi criado acima.
        unsafe { libc::close(ruleset_fd) };
        outcome
    }

    fn apply_no_new_privs() -> Layer {
        // SAFETY: prctl sem ponteiros.
        let rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if rc != 0 {
            return Layer::Unavailable(format!("PR_SET_NO_NEW_PRIVS failed ({})", last_error()));
        }
        // SAFETY: prctl sem ponteiros.
        let check = unsafe { libc::prctl(libc::PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0) };
        if check == 1 {
            Layer::Applied("on".into())
        } else {
            Layer::Unavailable("no_new_privs did not stick".into())
        }
    }

    fn apply_core_limit() -> Layer {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `limit` vive durante a chamada.
        let rc = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &limit) };
        if rc == 0 {
            Layer::Applied("nodump".into())
        } else {
            Layer::Unavailable(format!("setrlimit(RLIMIT_CORE) failed ({})", last_error()))
        }
    }

    // ---- seccomp (x86_64) -------------------------------------------------

    #[cfg(target_arch = "x86_64")]
    mod seccomp {
        use super::{Layer, last_error};

        const AUDIT_ARCH_X86_64: u32 = 0xC000_003E;
        const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;
        const SECCOMP_RET_ALLOW: u32 = 0x7FFF_0000;
        const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
        const BPF_LD_W_ABS: u16 = 0x20;
        const BPF_JMP_JEQ_K: u16 = 0x15;
        const BPF_JMP_JGE_K: u16 = 0x35;
        const BPF_RET_K: u16 = 0x06;
        /// Chamadas com este bit pertencem à ABI x32: o filtro as mata.
        const X32_SYSCALL_BIT: u32 = 0x4000_0000;

        #[repr(C)]
        pub struct SockFilter {
            code: u16,
            jt: u8,
            jf: u8,
            k: u32,
        }

        #[repr(C)]
        struct SockFprog {
            len: u16,
            filter: *const SockFilter,
        }

        /// Chamadas negadas com `EPERM`. É uma lista de **negação**: o engine
        /// usa runtime assíncrono, threads e compilação JIT (YARA-X), e uma
        /// lista de permissão exaustiva é frágil demais para o ganho.
        pub fn denied() -> Vec<u32> {
            [
                // Rede e sockets (inclui Unix): o engine só fala por stdin/stdout.
                // `socketpair` fica LIBERADO de propósito: o driver de sinais do
                // tokio (ligado pela unificação de features quando o workspace
                // é compilado junto com o app) cria um par anônimo no início do
                // runtime e o engine entra em pânico sem ele. Um par anônimo é
                // IPC dentro do próprio processo: não alcança rede nem outro
                // processo, e `socket`, `connect`, `bind`, `listen` e `accept`
                // seguem negados.
                libc::SYS_socket,
                libc::SYS_connect,
                libc::SYS_bind,
                libc::SYS_listen,
                libc::SYS_accept,
                libc::SYS_accept4,
                // Executar outro programa ou espiar outro processo.
                libc::SYS_execve,
                libc::SYS_execveat,
                libc::SYS_ptrace,
                libc::SYS_process_vm_readv,
                libc::SYS_process_vm_writev,
                libc::SYS_pidfd_getfd,
                libc::SYS_kcmp,
                // Montagem, namespaces e raiz.
                libc::SYS_mount,
                libc::SYS_umount2,
                libc::SYS_pivot_root,
                libc::SYS_chroot,
                libc::SYS_unshare,
                libc::SYS_setns,
                libc::SYS_open_tree,
                libc::SYS_move_mount,
                libc::SYS_fsopen,
                libc::SYS_fsconfig,
                libc::SYS_fsmount,
                libc::SYS_fspick,
                // Kernel, módulos, eBPF e superfícies exóticas.
                libc::SYS_bpf,
                libc::SYS_perf_event_open,
                libc::SYS_kexec_load,
                libc::SYS_init_module,
                libc::SYS_finit_module,
                libc::SYS_delete_module,
                libc::SYS_reboot,
                libc::SYS_swapon,
                libc::SYS_swapoff,
                libc::SYS_userfaultfd,
                libc::SYS_keyctl,
                libc::SYS_add_key,
                libc::SYS_request_key,
                libc::SYS_fanotify_init,
                // `io_uring` executa E/S fora do alcance do filtro.
                libc::SYS_io_uring_setup,
                libc::SYS_io_uring_enter,
                libc::SYS_io_uring_register,
                // Abrir arquivo por handle contorna o Landlock de caminho.
                libc::SYS_open_by_handle_at,
                libc::SYS_name_to_handle_at,
            ]
            .iter()
            .map(|&number| number as u32)
            .collect()
        }

        /// Monta o programa BPF clássico: confere a arquitetura, mata chamadas
        /// x32, nega as da lista e libera o resto.
        pub fn program(denied: &[u32]) -> Vec<SockFilter> {
            let n = denied.len();
            let stmt = |code: u16, k: u32| SockFilter {
                code,
                jt: 0,
                jf: 0,
                k,
            };
            let jump = |code: u16, k: u32, jt: u8, jf: u8| SockFilter { code, jt, jf, k };
            let mut program = Vec::with_capacity(n + 8);
            program.push(stmt(BPF_LD_W_ABS, 4)); // seccomp_data.arch
            program.push(jump(BPF_JMP_JEQ_K, AUDIT_ARCH_X86_64, 1, 0));
            program.push(stmt(BPF_RET_K, SECCOMP_RET_KILL_PROCESS));
            program.push(stmt(BPF_LD_W_ABS, 0)); // seccomp_data.nr
            // Índices: 4 = JGE x32; 5..5+n = JEQ; 5+n = ALLOW; 6+n = DENY; 7+n = KILL.
            program.push(jump(BPF_JMP_JGE_K, X32_SYSCALL_BIT, (n + 2) as u8, 0));
            for (i, number) in denied.iter().enumerate() {
                program.push(jump(BPF_JMP_JEQ_K, *number, (n - i) as u8, 0));
            }
            program.push(stmt(BPF_RET_K, SECCOMP_RET_ALLOW));
            program.push(stmt(BPF_RET_K, SECCOMP_RET_ERRNO | libc::EPERM as u32));
            program.push(stmt(BPF_RET_K, SECCOMP_RET_KILL_PROCESS));
            program
        }

        pub fn apply() -> Layer {
            let denied = denied();
            if denied.len() > 200 {
                return Layer::Unavailable(
                    "seccomp deny list is too long for the jump offsets".into(),
                );
            }
            let program = program(&denied);
            let fprog = SockFprog {
                len: program.len() as u16,
                filter: program.as_ptr(),
            };
            // SAFETY: `fprog` e `program` vivem durante a chamada; o filtro é
            // um BPF clássico bem formado.
            let rc = unsafe {
                libc::prctl(
                    libc::PR_SET_SECCOMP,
                    libc::SECCOMP_MODE_FILTER as libc::c_ulong,
                    &fprog as *const SockFprog,
                    0,
                    0,
                )
            };
            if rc != 0 {
                return Layer::Unavailable(format!(
                    "seccomp filter was refused ({})",
                    last_error()
                ));
            }
            Layer::Applied(format!("on({} denied)", denied.len()))
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn the_program_is_well_formed_and_jumps_stay_inside_it() {
                let denied = denied();
                let program = program(&denied);
                let n = denied.len();
                assert_eq!(program.len(), n + 8);
                // A última instrução mata; a anterior nega com EPERM; a anterior libera.
                assert_eq!(program[n + 7].k, SECCOMP_RET_KILL_PROCESS);
                assert_eq!(program[n + 6].k, SECCOMP_RET_ERRNO | libc::EPERM as u32);
                assert_eq!(program[n + 5].k, SECCOMP_RET_ALLOW);
                // Cada JEQ cai exatamente na instrução DENY (índice n + 6).
                for (i, instruction) in program.iter().enumerate().skip(5).take(n) {
                    assert_eq!(i + 1 + instruction.jt as usize, n + 6, "JEQ #{i}");
                }
                // O JGE do x32 cai na instrução KILL (índice n + 7).
                assert_eq!(4 + 1 + program[4].jt as usize, n + 7);
            }

            #[test]
            fn the_deny_list_has_no_duplicates_and_covers_the_essentials() {
                let denied = denied();
                let unique: std::collections::BTreeSet<_> = denied.iter().collect();
                assert_eq!(unique.len(), denied.len(), "sem números repetidos");
                for essential in [
                    libc::SYS_socket,
                    libc::SYS_connect,
                    libc::SYS_execve,
                    libc::SYS_ptrace,
                    libc::SYS_mount,
                    libc::SYS_bpf,
                    libc::SYS_io_uring_setup,
                ] {
                    assert!(denied.contains(&(essential as u32)));
                }
                // Chamadas que o engine usa não podem estar negadas.
                for needed in [
                    libc::SYS_read,
                    libc::SYS_write,
                    libc::SYS_openat,
                    libc::SYS_mmap,
                    libc::SYS_mprotect,
                    libc::SYS_futex,
                    libc::SYS_clone,
                    libc::SYS_epoll_wait,
                    // Usada pelo driver de sinais do tokio (ver `denied`).
                    libc::SYS_socketpair,
                ] {
                    assert!(!denied.contains(&(needed as u32)));
                }
            }
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    mod seccomp {
        use super::Layer;
        pub fn apply() -> Layer {
            Layer::Unavailable(
                "the seccomp filter is implemented for x86_64 only (Landlock still applies)".into(),
            )
        }
    }

    pub fn apply(policy: Policy, plan: &Plan) -> Status {
        // Ordem: no_new_privs primeiro (exigido por seccomp sem privilégio),
        // depois Landlock (sistema de arquivos), por fim seccomp (chamadas).
        // `policy` só é carregado no Status; o gate `Required` acontece em
        // `apply` (nível acima), então aqui não há uso — prefixo evita o
        // aviso sem descartar silenciosamente.
        let _policy = policy;
        let core_dumps = apply_core_limit();
        let no_new_privs = apply_no_new_privs();
        let landlock = apply_landlock(plan);
        let seccomp = if no_new_privs.is_applied() {
            seccomp::apply()
        } else {
            Layer::Unavailable("seccomp needs no_new_privs".into())
        };
        Status {
            policy,
            no_new_privs,
            landlock,
            seccomp,
            core_dumps,
            job: Layer::NotApplicable,
        }
    }
}

/// Windows: Job Object. O engine se coloca num job **antes** de ler qualquer
/// entrada (o supervisor só envia o pedido depois de iniciar o processo), com:
///
/// - teto de memória por processo (o mesmo que `RLIMIT_AS` faz no Linux; vem do
///   supervisor em `DFIR_ENGINE_MEMORY_LIMIT_BYTES`);
/// - no máximo 1 processo ativo no job: o engine **não cria processos filhos**
///   (o equivalente ao `execve` negado do seccomp);
/// - sem acesso à interface do usuário (área de transferência, desktop,
///   configurações de tela, atoms globais, handles de outras janelas);
/// - encerramento se o job for fechado ou houver exceção não tratada.
///
/// O que o Windows **não** recebe: confinamento do sistema de arquivos e da
/// rede (seria necessário AppContainer ou token restrito, ainda não feito e
/// não verificado). As camadas correspondentes ficam `Unavailable` e o job
/// registra o aviso `SANDBOX_NOT_FULLY_APPLIED` (ADR-0024).
#[cfg(windows)]
mod platform {
    use super::{Layer, Plan, Policy, Status};
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
        JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_UILIMIT_DESKTOP,
        JOB_OBJECT_UILIMIT_DISPLAYSETTINGS, JOB_OBJECT_UILIMIT_EXITWINDOWS,
        JOB_OBJECT_UILIMIT_GLOBALATOMS, JOB_OBJECT_UILIMIT_HANDLES,
        JOB_OBJECT_UILIMIT_READCLIPBOARD, JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS,
        JOB_OBJECT_UILIMIT_WRITECLIPBOARD, JOBOBJECT_BASIC_UI_RESTRICTIONS,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectBasicUIRestrictions,
        JobObjectExtendedLimitInformation, SetInformationJobObject,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    /// Teto de memória informado pelo supervisor (bytes).
    const MEMORY_LIMIT_ENV: &str = "DFIR_ENGINE_MEMORY_LIMIT_BYTES";

    fn apply_job() -> Layer {
        // SAFETY: todas as chamadas recebem ponteiros para structs locais
        // válidos durante a chamada; o handle do job é fechado ao final (o job
        // continua vivo enquanto o processo estiver nele).
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return Layer::Unavailable(format!("CreateJobObject failed ({})", GetLastError()));
            }
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
            limits.BasicLimitInformation.ActiveProcessLimit = 1;
            let memory_limit = std::env::var(MEMORY_LIMIT_ENV)
                .ok()
                .and_then(|raw| raw.trim().parse::<usize>().ok())
                .filter(|bytes| *bytes > 0);
            if let Some(bytes) = memory_limit {
                limits.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
                limits.ProcessMemoryLimit = bytes;
            }
            let ok = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if ok == 0 {
                let code = GetLastError();
                CloseHandle(job);
                return Layer::Unavailable(format!(
                    "SetInformationJobObject(limits) failed ({code})"
                ));
            }
            let ui = JOBOBJECT_BASIC_UI_RESTRICTIONS {
                UIRestrictionsClass: JOB_OBJECT_UILIMIT_DESKTOP
                    | JOB_OBJECT_UILIMIT_DISPLAYSETTINGS
                    | JOB_OBJECT_UILIMIT_EXITWINDOWS
                    | JOB_OBJECT_UILIMIT_GLOBALATOMS
                    | JOB_OBJECT_UILIMIT_HANDLES
                    | JOB_OBJECT_UILIMIT_READCLIPBOARD
                    | JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS
                    | JOB_OBJECT_UILIMIT_WRITECLIPBOARD,
            };
            let ok = SetInformationJobObject(
                job,
                JobObjectBasicUIRestrictions,
                &ui as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>() as u32,
            );
            if ok == 0 {
                let code = GetLastError();
                CloseHandle(job);
                return Layer::Unavailable(format!("SetInformationJobObject(ui) failed ({code})"));
            }
            if AssignProcessToJobObject(job, GetCurrentProcess()) == 0 {
                let code = GetLastError();
                CloseHandle(job);
                return Layer::Unavailable(format!("AssignProcessToJobObject failed ({code})"));
            }
            // O handle fica aberto de propósito: fechá-lo com KILL_ON_JOB_CLOSE
            // encerraria o próprio engine. Ele vive até o processo terminar.
            // F1-23: `let _ =` aqui é intencional e documentado; mantemos o
            // descarte explícito com o comentário acima.
            drop(job);
        }
        Layer::Applied(if std::env::var(MEMORY_LIMIT_ENV).is_ok() {
            "limits+mem".into()
        } else {
            "limits".into()
        })
    }

    pub fn apply(policy: Policy, _plan: &Plan) -> Status {
        let missing = |what: &str| {
            Layer::Unavailable(format!(
                "{what} is not implemented for Windows (the engine still runs as a separate process with a reduced environment and a single pinned input file)"
            ))
        };
        Status {
            policy,
            no_new_privs: Layer::NotApplicable,
            landlock: missing("file access confinement"),
            seccomp: missing("system call and network filtering"),
            core_dumps: Layer::NotApplicable,
            job: apply_job(),
        }
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod platform {
    use super::{Layer, Plan, Policy, Status};

    pub fn apply(policy: Policy, _plan: &Plan) -> Status {
        let reason = format!(
            "no OS-level containment is implemented for {} (the engine still runs as a separate process with a reduced environment and a single pinned input file)",
            std::env::consts::OS
        );
        Status {
            policy,
            no_new_privs: Layer::Unavailable(reason.clone()),
            landlock: Layer::Unavailable(reason.clone()),
            seccomp: Layer::Unavailable(reason.clone()),
            core_dumps: Layer::Unavailable(reason.clone()),
            job: Layer::Unavailable(reason),
        }
    }
}

/// Provas feitas **de dentro** do processo já contido (`--sandbox-selftest`):
/// tenta o que o sandbox deve negar e o que deve permitir, e devolve o
/// `errno` de cada tentativa. Usado pelo teste de integração para provar que a
/// contenção é real, não só aplicada.
#[cfg(target_os = "linux")]
pub fn selftest(pinned_file: Option<&std::path::Path>) -> serde_json::Value {
    use serde_json::json;
    use std::io::Read;

    fn errno_of<T>(result: std::io::Result<T>) -> serde_json::Value {
        match result {
            Ok(_) => json!(0),
            Err(error) => json!(error.raw_os_error().unwrap_or(-1)),
        }
    }
    fn syscall_errno(rc: libc::c_long) -> serde_json::Value {
        if rc >= 0 {
            json!(0)
        } else {
            json!(std::io::Error::last_os_error().raw_os_error().unwrap_or(-1))
        }
    }

    let read_pinned = pinned_file.map(|path| {
        errno_of(std::fs::File::open(path).and_then(|mut file| {
            let mut buffer = [0_u8; 16];
            file.read(&mut buffer)
        }))
    });
    let read_other = errno_of(std::fs::File::open("/etc/passwd"));
    let read_dir_other = errno_of(std::fs::read_dir("/etc").map(|mut entries| entries.next()));
    let create_file = errno_of(std::fs::File::create("/tmp/dfir-sandbox-selftest.txt"));
    // SAFETY: as chamadas abaixo não recebem ponteiros além de literais C
    // válidos e devem falhar pelo filtro antes de qualquer efeito.
    let tcp_socket =
        syscall_errno(unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) } as libc::c_long);
    let unix_socket =
        syscall_errno(unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) } as libc::c_long);
    // Par anônimo (usado pelo runtime): permitido, e só ele.
    let socketpair = {
        let mut fds = [0 as libc::c_int; 2];
        // SAFETY: `fds` tem espaço para os dois descritores.
        let rc = unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) };
        let result = syscall_errno(rc as libc::c_long);
        if rc == 0 {
            // SAFETY: descritores recém-criados acima.
            unsafe {
                libc::close(fds[0]);
                libc::close(fds[1]);
            }
        }
        result
    };
    // O `execve` roda num processo filho: sem contenção ele **substituiria**
    // este processo por `/bin/true`. O filho sai com o `errno` se o exec falhar.
    let exec = {
        let program = c"/bin/true";
        let argv = [program.as_ptr(), std::ptr::null()];
        let envp = [std::ptr::null()];
        // SAFETY: `fork` numa thread única (o selftest roda antes do runtime);
        // o filho só chama `execve` e `_exit`, ambas seguras após o fork.
        let pid = unsafe { libc::fork() };
        if pid == 0 {
            // SAFETY: `argv`/`envp` terminam em nulo e `program` é uma string C.
            unsafe {
                libc::execve(program.as_ptr(), argv.as_ptr(), envp.as_ptr());
                let errno = std::io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(255);
                libc::_exit(errno);
            }
        }
        if pid < 0 {
            json!(-1)
        } else {
            let mut status = 0;
            // SAFETY: `pid` é o filho recém-criado; `status` vive na chamada.
            unsafe { libc::waitpid(pid, &mut status, 0) };
            if libc::WIFEXITED(status) {
                json!(libc::WEXITSTATUS(status))
            } else {
                json!(-1)
            }
        }
    };
    let ptrace = syscall_errno(unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) });
    // O que PRECISA continuar funcionando: alocar, criar thread e medir tempo.
    let thread_ok = std::thread::spawn(|| 41 + 1)
        .join()
        .map(|value| value == 42)
        .unwrap_or(false);
    let allocation_ok = vec![7_u8; 8 * 1024 * 1024].iter().all(|byte| *byte == 7);

    json!({
        "read_pinned_file": read_pinned,
        "read_other_file": read_other,
        "list_other_dir": read_dir_other,
        "create_file": create_file,
        "tcp_socket": tcp_socket,
        "unix_socket": unix_socket,
        "socketpair": socketpair,
        "execve": exec,
        "ptrace": ptrace,
        "thread_ok": thread_ok,
        "allocation_ok": allocation_ok,
    })
}

#[cfg(not(target_os = "linux"))]
pub fn selftest(_pinned_file: Option<&std::path::Path>) -> serde_json::Value {
    serde_json::json!({ "unsupported": std::env::consts::OS })
}

#[cfg(test)]
mod tests {
    use super::{Layer, Plan, Policy, Status, apply};

    #[test]
    fn policy_parsing_accepts_the_documented_names_and_nothing_else() {
        assert_eq!(Policy::parse("required"), Some(Policy::Required));
        assert_eq!(Policy::parse(" Best-Effort "), Some(Policy::BestEffort));
        assert_eq!(Policy::parse("off"), Some(Policy::Off));
        assert_eq!(Policy::parse("yes"), None);
        assert_eq!(Policy::parse(""), None);
    }

    #[test]
    fn off_applies_nothing_and_says_so() {
        let status = apply(Policy::Off, &Plan::default()).expect("off never fails");
        assert!(!status.fully_applied());
        assert_eq!(status.landlock, Layer::Skipped);
        assert_eq!(
            status.summary(),
            "policy=off;nnp=skipped;landlock=skipped;seccomp=skipped;core=skipped;job=skipped"
        );
    }

    #[test]
    fn the_summary_never_carries_paths_or_reasons() {
        let status = Status {
            policy: Policy::BestEffort,
            no_new_privs: Layer::Applied("on".into()),
            landlock: Layer::Unavailable("Landlock is not available in /secret/path".into()),
            seccomp: Layer::Applied("on(45 denied)".into()),
            core_dumps: Layer::Applied("nodump".into()),
            job: Layer::NotApplicable,
        };
        let summary = status.summary();
        assert!(!summary.contains("/secret"), "{summary}");
        assert!(summary.contains("landlock=unavailable"));
        assert_eq!(status.missing().len(), 1);
        assert!(!status.fully_applied());
    }

    #[test]
    fn a_layer_that_does_not_exist_on_this_os_is_not_a_failure() {
        let status = Status {
            policy: Policy::Required,
            no_new_privs: Layer::NotApplicable,
            landlock: Layer::Applied("abi7".into()),
            seccomp: Layer::Applied("on".into()),
            core_dumps: Layer::NotApplicable,
            job: Layer::NotApplicable,
        };
        assert!(
            status.fully_applied(),
            "NotApplicable não conta como ausência"
        );
        assert!(status.summary().contains("job=na"));
        assert!(status.missing().is_empty());
    }
}
