use crate::{
    db::AppState,
    models::{
        CommandError, CommandErrorCode, OpenAiConnectionResultDto, OpenAiCredentialStatusDto,
    },
};

pub const OPENAI_CREDENTIAL_ALIAS: &str = "secret://openai/default";
const KEYRING_SERVICE: &str = "com.sangjinsu.cole";
const KEYRING_ACCOUNT: &str = "openai/default";

pub trait CredentialStore: Send + Sync {
    fn set(&self, secret: &str) -> Result<(), String>;
    fn get(&self) -> Result<Option<String>, String>;
    fn delete(&self) -> Result<(), String>;
}

pub struct KeyringCredentialStore;

impl KeyringCredentialStore {
    pub fn new() -> Self {
        Self
    }

    fn entry(&self) -> Result<keyring::Entry, String> {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
            .map_err(|_| "OS credential storage is unavailable".to_string())
    }
}

impl Default for KeyringCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn set(&self, secret: &str) -> Result<(), String> {
        self.entry()?
            .set_password(secret)
            .map_err(|_| "failed to save the OpenAI credential".to_string())
    }

    fn get(&self) -> Result<Option<String>, String> {
        match self.entry()?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err("failed to read the OpenAI credential".to_string()),
        }
    }

    fn delete(&self) -> Result<(), String> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("failed to delete the OpenAI credential".to_string()),
        }
    }
}

pub fn set_openai_api_key(
    state: &AppState,
    api_key: &str,
) -> Result<OpenAiCredentialStatusDto, CommandError> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(CommandError::new(
            CommandErrorCode::ValidationError,
            "an OpenAI API key is required",
        ));
    }
    let (_, credential_version) = state.with_db(|db| db.bump_openai_credential_version())?;
    state
        .credential_store()
        .set(api_key)
        .map_err(credential_error)?;
    Ok(OpenAiCredentialStatusDto {
        configured: true,
        alias: Some(OPENAI_CREDENTIAL_ALIAS.to_string()),
        credential_version,
    })
}

pub fn get_openai_credential_status(
    state: &AppState,
) -> Result<OpenAiCredentialStatusDto, CommandError> {
    let (alias, credential_version) = state.with_db(|db| db.openai_credential_metadata())?;
    let configured = state
        .credential_store()
        .get()
        .map_err(credential_error)?
        .is_some();
    Ok(OpenAiCredentialStatusDto {
        configured,
        alias: configured.then_some(alias),
        credential_version,
    })
}

pub fn delete_openai_api_key(state: &AppState) -> Result<OpenAiCredentialStatusDto, CommandError> {
    let (_, credential_version) = state.with_db(|db| db.bump_openai_credential_version())?;
    state
        .credential_store()
        .delete()
        .map_err(credential_error)?;
    Ok(OpenAiCredentialStatusDto {
        configured: false,
        alias: None,
        credential_version,
    })
}

pub async fn test_openai_connection(
    state: &AppState,
) -> Result<OpenAiConnectionResultDto, CommandError> {
    let api_key = state.credential_store().get().map_err(credential_error)?;
    let Some(api_key) = api_key else {
        return Ok(OpenAiConnectionResultDto {
            ok: false,
            message: "No OpenAI credential is configured.".to_string(),
        });
    };
    match state.analysis_provider().test_connection(&api_key).await {
        Ok(model) => Ok(OpenAiConnectionResultDto {
            ok: true,
            message: format!("Connected to OpenAI ({model})."),
        }),
        Err(_) => Ok(OpenAiConnectionResultDto {
            ok: false,
            message: "OpenAI connection test failed.".to_string(),
        }),
    }
}

fn credential_error(message: String) -> CommandError {
    CommandError::new(CommandErrorCode::CredentialError, message)
}
