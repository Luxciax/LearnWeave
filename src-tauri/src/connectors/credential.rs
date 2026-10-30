use keyring::{Entry, Error};

const SERVICE: &str = "LearnWeave";
const LEGACY_SERVICE: &str = "ZhiForge";

fn account(connector_id: &str) -> String {
    format!("connector:{}:api-key", connector_id.trim().to_ascii_lowercase())
}

fn entry(service: &str, connector_id: &str) -> Result<Entry, String> {
    Entry::new(service, &account(connector_id))
        .map_err(|error| format!("connector credential store unavailable: {error}"))
}

pub fn load(connector_id: &str) -> Result<String, String> {
    match entry(SERVICE, connector_id)?.get_password() {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        Ok(_) | Err(Error::NoEntry) => match entry(LEGACY_SERVICE, connector_id)?.get_password() {
            Ok(value) => Ok(value),
            Err(Error::NoEntry) => Ok(String::new()),
            Err(error) => Err(format!("failed to read legacy connector API key: {error}")),
        },
        Err(error) => Err(format!("failed to read connector API key: {error}")),
    }
}

#[tauri::command]
pub fn connector_api_key_configured(connector_id: String) -> Result<bool, String> {
    Ok(!load(&connector_id)?.trim().is_empty())
}

#[tauri::command]
pub fn connector_save_api_key(connector_id: String, api_key: String) -> Result<(), String> {
    let connector_id = connector_id.trim();
    if connector_id.is_empty() || connector_id.len() > 80 {
        return Err("invalid connector id".into());
    }
    let item = entry(SERVICE, connector_id)?;
    if api_key.trim().is_empty() {
        match item.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(error) => Err(format!("failed to clear connector API key: {error}")),
        }
    } else {
        item.set_password(api_key.trim())
            .map_err(|error| format!("failed to save connector API key: {error}"))
    }
}
