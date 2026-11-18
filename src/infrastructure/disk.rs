//! Espaço livre em disco (ADR-0021).
//!
//! Sem teto de saída (ADR-0020), o que protege o volume de dados de um arquivo
//! hostil ou grande demais é esta guarda: não importa nem começa uma análise
//! com o disco quase cheio, e uma análise em andamento é interrompida (com
//! erro explícito, nunca em silêncio) se o espaço livre cair abaixo da margem.
//!
//! A leitura do espaço livre existe para Unix (`statvfs`). Em outras
//! plataformas o resultado é [`SpaceCheck::Unknown`], que **não bloqueia**:
//! a guarda fica desligada ali em vez de impedir o uso.

use std::{io, path::Path};

/// Margem mínima de espaço livre por padrão: 1 GiB.
pub const DEFAULT_MIN_FREE_DISK_BYTES: u64 = 1024 * 1024 * 1024;
/// Teto do valor configurável (1 TiB).
pub const MAX_MIN_FREE_DISK_BYTES: u64 = 1024 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceCheck {
    Enough,
    Low {
        free: u64,
        required: u64,
    },
    /// Não foi possível medir (plataforma sem suporte ou falha da chamada).
    Unknown,
}

/// Decisão pura, testável sem tocar no disco.
pub fn evaluate(free: Option<u64>, required: u64) -> SpaceCheck {
    match free {
        None => SpaceCheck::Unknown,
        Some(free) if free >= required => SpaceCheck::Enough,
        Some(free) => SpaceCheck::Low { free, required },
    }
}

/// Verifica o volume que contém `path`. `required == 0` desliga a guarda.
pub fn check_path(path: &Path, required: u64) -> SpaceCheck {
    if required == 0 {
        return SpaceCheck::Enough;
    }
    evaluate(available_space(path).ok(), required)
}

/// Mensagem acionável quando falta espaço; `None` se está tudo bem (ou não
/// foi possível medir).
pub fn shortage_message(path: &Path, required: u64) -> Option<String> {
    match check_path(path, required) {
        SpaceCheck::Low { free, required } => Some(format!(
            "not enough free disk space on the data volume: {} MiB free, {} MiB required (the margin is set by DFIR_MIN_FREE_DISK_BYTES; free some space or move DFIR_DATA_ROOT to a larger volume)",
            free / (1024 * 1024),
            required / (1024 * 1024)
        )),
        SpaceCheck::Enough | SpaceCheck::Unknown => None,
    }
}

/// Bytes disponíveis a um usuário comum no volume que contém `path`.
// Os tipos de `f_bavail` e `f_frsize` variam por plataforma (u64 no Linux
// x86_64; u32 ou `c_ulong` em outras): o cast é necessário fora daqui.
#[allow(clippy::unnecessary_cast)]
#[cfg(unix)]
pub fn available_space(path: &Path) -> io::Result<u64> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains a NUL byte"))?;
    // SAFETY: `statvfs` é uma struct de dados simples (zerada é válida) e
    // `c_path` é uma string C válida que vive até o fim da chamada.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64))
}

#[cfg(not(unix))]
pub fn available_space(_path: &Path) -> io::Result<u64> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_is_pure_and_conservative_about_unknowns() {
        assert_eq!(evaluate(Some(10), 10), SpaceCheck::Enough);
        assert_eq!(evaluate(Some(11), 10), SpaceCheck::Enough);
        assert_eq!(
            evaluate(Some(9), 10),
            SpaceCheck::Low {
                free: 9,
                required: 10
            }
        );
        assert_eq!(evaluate(None, 10), SpaceCheck::Unknown, "não bloqueia");
    }

    #[test]
    fn a_zero_margin_disables_the_guard() {
        assert_eq!(
            check_path(Path::new("/definitely/not/here"), 0),
            SpaceCheck::Enough
        );
    }

    #[cfg(unix)]
    #[test]
    fn real_volume_reports_space_and_an_absurd_margin_is_refused() {
        let dir = tempfile::tempdir().expect("temp dir");
        let free = available_space(dir.path()).expect("statvfs works on a temp dir");
        assert!(free > 0);
        assert!(matches!(
            check_path(dir.path(), u64::MAX),
            SpaceCheck::Low { .. }
        ));
        let message = shortage_message(dir.path(), u64::MAX).expect("shortage message");
        assert!(message.contains("DFIR_MIN_FREE_DISK_BYTES"), "{message}");
        assert!(shortage_message(dir.path(), 1).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn a_missing_path_is_unknown_not_a_refusal() {
        assert_eq!(
            check_path(Path::new("/definitely/not/here"), 1),
            SpaceCheck::Unknown
        );
    }
}
