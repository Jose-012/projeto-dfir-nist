//! Seleção do backend de diálogo nativo.
//!
//! A escolha é explícita pela variável de ambiente
//! `DFIR_FILE_DIALOG_BACKEND=auto|zenity|rfd`:
//!
//! - `auto` (padrão): no Linux, usa [`Backend::Zenity`] quando uma sonda
//!   curta e cacheada confirma que o binário `zenity` existe; caso contrário,
//!   cai para [`Backend::Rfd`]. Nas demais plataformas, sempre [`Backend::Rfd`].
//! - `zenity`: sempre o `zenity` direto. Em outra plataforma, o backend
//!   devolve `Picked::Unavailable` com diagnóstico explícito.
//! - `rfd`: sempre o caminho legado `rfd` (portal primeiro no Linux), com a
//!   limitação residual documentada em [`super::rfd_legacy`].
//!
//! Um valor **inválido** não é ignorado: devolve erro específico, para que
//! erro de digitação não pareça configuração padrão (mesma régua de
//! `DFIR_ENGINE_SANDBOX` em `src/config.rs`). A preferência é validada na UI,
//! mas a sonda de `auto` só executa na thread do diálogo.

use super::{Picked, Request, zenity};

/// Variável de ambiente que escolhe o backend.
pub const BACKEND_ENV: &str = "DFIR_FILE_DIALOG_BACKEND";

/// Backend de diálogo nativo efetivamente em uso.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// `zenity` direto via `std::process::Command` (sem shell, sem timeout
    /// enquanto o operador escolhe, saída limitada).
    Zenity,
    /// `rfd` (portal primeiro no Linux; demais plataformas). Limitação
    /// residual da API documentada em [`super::rfd_legacy`].
    Rfd,
}

impl Backend {
    /// Rótulo curto para logs (`backend`), sem dados do diálogo.
    pub fn label(self) -> &'static str {
        match self {
            Self::Zenity => "zenity",
            Self::Rfd => "rfd",
        }
    }

    /// Executa a requisição de forma bloqueante (roda na thread do diálogo).
    /// `child_slot` recebe o processo filho quando o backend é `zenity`,
    /// para o `Drop` do [`super::FilePicker`] poder matar e colher.
    pub fn run(self, request: Request, child_slot: zenity::ChildSlot) -> Picked {
        match self {
            Self::Zenity => zenity::run(request, child_slot),
            Self::Rfd => super::rfd_legacy::run(request),
        }
    }
}

/// Preferência barata de validar, antes de qualquer subprocesso ou espera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preference {
    Auto,
    Zenity,
    Rfd,
}

impl Preference {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Zenity => "zenity",
            Self::Rfd => "rfd",
        }
    }

    /// Resolve `auto` somente na thread do diálogo. Valores explícitos nunca
    /// executam a sonda.
    pub fn resolve(self) -> Backend {
        self.resolve_with(zenity::probe_available)
    }

    fn resolve_with(self, probe: impl FnOnce() -> bool) -> Backend {
        match self {
            Self::Auto if cfg!(target_os = "linux") && probe() => Backend::Zenity,
            Self::Auto => Backend::Rfd,
            Self::Zenity => Backend::Zenity,
            Self::Rfd => Backend::Rfd,
        }
    }
}

/// Lê a preferência a partir de `DFIR_FILE_DIALOG_BACKEND` sem executar sonda.
///
/// Valor ausente ou vazio = `auto`. Valor inválido = `Err` com a lista de
/// valores aceitos.
pub fn from_env() -> Result<Preference, String> {
    preference_from(std::env::var(BACKEND_ENV).ok())
}

/// Parte testável de [`from_env`], sem depender do ambiente do processo.
pub(crate) fn preference_from(raw: Option<String>) -> Result<Preference, String> {
    let Some(raw) = raw else {
        return Ok(Preference::Auto);
    };
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "auto" => Ok(Preference::Auto),
        "zenity" => Ok(Preference::Zenity),
        "rfd" => Ok(Preference::Rfd),
        other => Err(format!(
            "{BACKEND_ENV}={other:?} é inválido; use um de: auto, zenity, rfd"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_or_empty_means_auto() {
        assert_eq!(
            preference_from(None).expect("auto"),
            preference_from(Some(String::new())).expect("vazio = auto"),
        );
        assert_eq!(
            preference_from(Some("  AUTO ".into())).expect("case/trim"),
            Preference::Auto,
        );
    }

    #[test]
    fn explicit_values_are_accepted() {
        assert_eq!(
            preference_from(Some("zenity".into())).expect("zenity"),
            Preference::Zenity
        );
        assert_eq!(
            preference_from(Some("RFD".into())).expect("rfd"),
            Preference::Rfd
        );
    }

    #[test]
    fn an_invalid_value_fails_specifically() {
        let err =
            preference_from(Some("kdialog".into())).expect_err("valor inválido precisa falhar");
        assert!(
            err.contains("DFIR_FILE_DIALOG_BACKEND") && err.contains("auto, zenity, rfd"),
            "erro deve nomear a variável e os valores aceitos: {err}"
        );
    }

    #[test]
    fn automatic_probe_is_deferred_until_resolution() {
        let preference = preference_from(Some("auto".into())).expect("auto");
        let mut probed = false;
        assert_eq!(
            preference.resolve_with(|| {
                probed = true;
                false
            }),
            Backend::Rfd
        );
        assert!(probed, "auto deve consultar a sonda somente ao resolver");

        let mut explicit_probe = false;
        assert_eq!(
            Preference::Zenity.resolve_with(|| {
                explicit_probe = true;
                false
            }),
            Backend::Zenity
        );
        assert!(!explicit_probe, "backend explícito não deve ser sondado");
    }
}
