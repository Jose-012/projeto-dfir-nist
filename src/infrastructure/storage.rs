use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct StoragePaths {
    pub db: PathBuf,
    pub runtime: PathBuf,
    pub evidence: PathBuf,
    pub staging: PathBuf,
    pub sanitized: PathBuf,
    pub exports: PathBuf,
    pub quarantine: PathBuf,
    /// Logs da aplicação (ADR-0022).
    pub logs: PathBuf,
    /// Cópias verificadas do banco antes de migration (RF-032).
    pub backups: PathBuf,
}

impl StoragePaths {
    pub fn from_root(root: &Path) -> Self {
        Self {
            db: root.join("db"),
            runtime: root.join("runtime"),
            evidence: root.join("evidence"),
            staging: root.join("staging"),
            sanitized: root.join("sanitized"),
            exports: root.join("exports"),
            quarantine: root.join("quarantine"),
            logs: root.join("logs"),
            backups: root.join("backups"),
        }
    }

    pub fn ensure(&self) -> Result<(), io::Error> {
        for dir in [
            &self.db,
            &self.runtime,
            &self.evidence,
            &self.staging,
            &self.sanitized,
            &self.exports,
            &self.quarantine,
            &self.logs,
            &self.backups,
        ] {
            fs::create_dir_all(dir)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
            }
        }
        Ok(())
    }

    /// Fase 0 accepts only logical relative references. Physical paths remain an application concern.
    /// Callers that later open files must apply no-follow/symlink-safe filesystem policy.
    pub fn logical_path(&self, reference: &str) -> Result<PathBuf, io::Error> {
        if reference.is_empty()
            || reference.contains('\0')
            || reference.contains('\\')
            || reference.contains(':')
            || Path::new(reference).is_absolute()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid logical reference",
            ));
        }
        let mut parts = reference.split('/');
        let root_name = parts.next().unwrap_or_default();
        let base = match root_name {
            "evidence" => &self.evidence,
            "staging" => &self.staging,
            "sanitized" => &self.sanitized,
            "exports" => &self.exports,
            "quarantine" => &self.quarantine,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unknown storage root",
                ));
            }
        };
        let remainder: Vec<&str> = parts.collect();
        if remainder.is_empty()
            || remainder
                .iter()
                .any(|p| p.is_empty() || *p == "." || *p == "..")
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid logical reference",
            ));
        }
        Ok(base.join(remainder.join("/")))
    }
}

#[cfg(test)]
mod tests {
    use super::StoragePaths;
    use std::path::PathBuf;

    #[test]
    fn logical_reference_rejects_traversal() {
        let paths = StoragePaths {
            db: PathBuf::from("/data/db"),
            runtime: PathBuf::from("/data/runtime"),
            evidence: PathBuf::from("/data/evidence"),
            staging: PathBuf::from("/data/staging"),
            sanitized: PathBuf::from("/data/sanitized"),
            exports: PathBuf::from("/data/exports"),
            quarantine: PathBuf::from("/data/quarantine"),
            logs: PathBuf::from("/data/logs"),
            backups: PathBuf::from("/data/backups"),
        };
        assert!(paths.logical_path("evidence/ev-1/file.bin").is_ok());
        assert!(paths.logical_path("evidence/../db/file.bin").is_err());
        assert!(paths.logical_path("evidence/file:stream").is_err());
        assert!(paths.logical_path("/etc/passwd").is_err());
    }
}
