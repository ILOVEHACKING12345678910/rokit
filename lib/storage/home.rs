use std::env::var;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::result::{AftmanError, AftmanResult};

use super::{InstallCache, ToolStorage, TrustCache};

/**
    Aftman's home directory - this is where Aftman stores its
    configuration, tools, and other data. Can be cheaply cloned
    while still referring to the same underlying data.

    By default, this is `$HOME/.aftman`, but can be overridden
    by setting the `AFTMAN_ROOT` environment variable.
*/
#[derive(Debug, Clone)]
pub struct Home {
    path: Arc<Path>,
    did_save: Arc<AtomicBool>,
    trust_cache: TrustCache,
    install_cache: InstallCache,
    tool_storage: ToolStorage,
}

impl Home {
    /**
        Creates a new `Home` from the given path.
    */
    async fn load_from_path(path: impl Into<PathBuf>) -> AftmanResult<Self> {
        let path: Arc<Path> = path.into().into();
        let did_save = Arc::new(AtomicBool::new(false));

        let (trust_cache, install_cache, tool_storage) = tokio::try_join!(
            TrustCache::load(&path),
            InstallCache::load(&path),
            ToolStorage::load(&path),
        )?;

        Ok(Self {
            path,
            did_save,
            trust_cache,
            install_cache,
            tool_storage,
        })
    }

    /**
        Creates a new `Home` from the environment.

        This will read, and if necessary, create the Aftman home directory
        and its contents - including trust storage, tools storage, etc.

        If the `AFTMAN_ROOT` environment variable is set, this will use
        that as the home directory. Otherwise, it will use `$HOME/.aftman`.
    */
    pub async fn load_from_env() -> AftmanResult<Self> {
        Ok(match var("AFTMAN_ROOT") {
            Ok(root_str) => Self::load_from_path(root_str).await?,
            Err(_) => {
                let path = dirs::home_dir()
                    .ok_or(AftmanError::HomeNotFound)?
                    .join(".aftman");

                Self::load_from_path(path).await?
            }
        })
    }

    /**
        Gets a reference to the path for this `Home`.
    */
    pub fn path(&self) -> &Path {
        &self.path
    }

    /**
        Returns a reference to the `TrustCache` for this `Home`.
    */
    pub fn trust_cache(&self) -> &TrustCache {
        &self.trust_cache
    }

    /**
        Returns a reference to the `InstallCache` for this `Home`.
    */
    pub fn install_cache(&self) -> &InstallCache {
        &self.install_cache
    }

    /**
        Returns a reference to the `ToolStorage` for this `Home`.
    */
    pub fn tool_storage(&self) -> &ToolStorage {
        &self.tool_storage
    }

    /**
        Saves the contents of this `Home` to disk.
    */
    pub async fn save(&self) -> AftmanResult<()> {
        tokio::try_join!(
            self.trust_cache.save(&self.path),
            self.install_cache.save(&self.path),
        )?;
        self.did_save.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/*
    Implement Drop with an error message if the Home was dropped
    without being saved - this should never happen since a Home
    should always be loaded once on startup and saved on shutdown
    in the CLI, but this detail may be missed during refactoring.

    In the future, if AsyncDrop ever becomes a thing, we can just
    force the save to happen in the Drop implementation instead.
*/
impl Drop for Home {
    fn drop(&mut self) {
        let is_last = Arc::strong_count(&self.path) <= 1;
        if is_last && !self.did_save.load(Ordering::SeqCst) {
            tracing::error!(
                "Aftman home was dropped without being saved!\
                \nChanges to trust, tools, and more may have been lost."
            )
        }
    }
}
