//! Montagem de `AppServices` sobre um diretório temporário, para os testes
//! de serviço.

use super::AppServices;
use crate::application::engine::EngineRunnerConfig;
use crate::config::AppConfig;
use crate::infrastructure::storage::StoragePaths;
use crate::persistence::db;
use std::{path::PathBuf, time::Duration};

pub(crate) struct Fixture {
    pub services: AppServices,
    _root: tempfile::TempDir,
}

pub(crate) async fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("temporary directory");
    let paths = StoragePaths::from_root(root.path());
    paths.ensure().expect("storage layout");
    let url = format!(
        "sqlite://{}",
        paths.db.join("service-test.sqlite").display()
    );
    let pool = db::connect(&url).await.expect("database connection");
    db::migrate(&pool).await.expect("migrations");
    let config = AppConfig {
        data_root: root.path().to_path_buf(),
        engine_path: PathBuf::from("dfir-engine"),
        engine_max_line_bytes: 1024 * 1024,
        engine_timeout: Duration::from_secs(5),
        engine_max_concurrent_jobs: 1,
        engine_max_memory_bytes: 0,
        max_upload_bytes: 1024 * 1024,
        min_free_disk_bytes: 0,
        engine_sandbox: None,
    };
    Fixture {
        services: AppServices {
            db: pool,
            paths,
            engine: EngineRunnerConfig::from_config(&config),
        },
        _root: root,
    }
}
