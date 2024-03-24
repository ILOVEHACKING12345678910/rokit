use std::{path::Path, str::FromStr};

use toml_edit::{DocumentMut, Formatted, Item, Value};

use crate::{
    result::{AftmanError, AftmanResult},
    sources::ArtifactProvider,
    util::{load_from_file_fallible, save_to_file},
};

const MANIFEST_FILE_NAME: &str = "auth.toml";
const MANIFEST_DEFAULT_CONTENTS: &str = r#"
# This file lists authentication tokens managed by Aftman, a cross-platform toolchain manager.
# For more information, see <|REPOSITORY_URL|>

# github = "ghp_tokenabcdef1234567890"
"#;

/**
    Authentication manifest file.

    Contains authentication tokens managed by Aftman.
*/
#[derive(Debug, Clone)]
pub struct AuthManifest {
    document: DocumentMut,
}

impl AuthManifest {
    /**
        Loads the manifest from the given directory, or creates a new one if it doesn't exist.

        If the manifest doesn't exist, a new one will be created with default contents and saved.

        See [`AuthManifest::load`] and [`AuthManifest::save`] for more information.
    */
    pub async fn load_or_create(dir: impl AsRef<Path>) -> AftmanResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        match load_from_file_fallible(path).await {
            Ok(manifest) => Ok(manifest),
            Err(AftmanError::FileNotFound(_)) => {
                let new = Self::default();
                new.save(dir).await?;
                Ok(new)
            }
            Err(e) => Err(e),
        }
    }

    /**
        Loads the manifest from the given directory.

        This will search for a file named `auth.toml` in the given directory.
    */
    pub async fn load(dir: impl AsRef<Path>) -> AftmanResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        load_from_file_fallible(path).await
    }

    /**
        Saves the manifest to the given directory.

        This will write the manifest to a file named `auth.toml` in the given directory.
    */
    pub async fn save(&self, dir: impl AsRef<Path>) -> AftmanResult<()> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        save_to_file(path, self.clone()).await
    }

    /**
        Checks if the manifest contains an authentication token for the given artifact provider.
    */
    pub fn has_token(&self, artifact_provider: ArtifactProvider) -> bool {
        self.document.contains_key(artifact_provider.as_str())
    }

    /**
        Gets the authentication token for the given artifact provider.

        Returns `None` if the token is not present.
    */
    pub fn get_token(&self, artifact_provider: ArtifactProvider) -> Option<String> {
        let token = self.document.get(artifact_provider.as_str())?;
        token.as_str().map(|s| s.to_string())
    }

    /**
        Sets the authentication token for the given artifact provider.

        Returns `true` if the token replaced an older
        one, `false` if an older token was not present.
    */
    pub fn set_token(
        &mut self,
        artifact_provider: ArtifactProvider,
        token: impl Into<String>,
    ) -> bool {
        let tab = self.document.as_table_mut();
        let old = tab.insert(
            artifact_provider.as_str(),
            Item::Value(Value::String(Formatted::new(token.into()))),
        );
        old.is_some()
    }
}

impl FromStr for AuthManifest {
    type Err = toml_edit::TomlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let document = s.parse::<DocumentMut>()?;
        Ok(Self { document })
    }
}

impl ToString for AuthManifest {
    fn to_string(&self) -> String {
        self.document.to_string()
    }
}

impl Default for AuthManifest {
    fn default() -> Self {
        let document = MANIFEST_DEFAULT_CONTENTS
            .replace("<|REPOSITORY_URL|>", env!("CARGO_PKG_REPOSITORY"))
            .parse::<DocumentMut>()
            .unwrap();
        Self { document }
    }
}
