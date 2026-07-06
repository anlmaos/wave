// Animawave - secrets.rs
// Copyright (C) 2025  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cell::Cell;
use std::collections::HashMap;

use crate::settings::settings_manager;
use crate::settings::Key;

thread_local! {
    static LAST_MODE: Cell<StorageMode> = const { Cell::new(StorageMode::GSettings) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMode {
    SecretService,
    GSettings,
}

const ITEM_LABEL: &str = "Animawave AI API Key";
const ATTR_SERVICE: &str = "animawave-service";
const ATTR_VALUE: &str = "animawave-ai-api-key";

/// Try to read the API key from Secret Service (GNOME Keyring / KWallet).
/// Returns `None` if Secret Service is unavailable or the key is not stored there.
async fn try_read_from_secret() -> Option<String> {
    let ss = secret_service::SecretService::connect(secret_service::EncryptionType::Dh)
        .await
        .ok()?;

    // First try searching directly (searches all collections including default)
    let result = ss
        .search_items(HashMap::from([(ATTR_SERVICE, ATTR_VALUE)]))
        .await
        .ok()?;

    // Check unlocked items first
    if let Some(item) = result.unlocked.first() {
        if let Ok(secret) = item.get_secret().await {
            if let Ok(value) = String::from_utf8(secret) {
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }

    // Try unlocking a locked item
    if let Some(item) = result.locked.first() {
        item.unlock().await.ok()?;
        let secret = item.get_secret().await.ok()?;
        let value = String::from_utf8(secret).ok()?;
        if value.is_empty() {
            return None;
        }
        return Some(value);
    }

    None
}

/// Try to store the API key in Secret Service.
/// Returns `true` on success, `false` if Secret Service is unavailable.
async fn try_store_in_secret(key: &str) -> bool {
    let Ok(ss) = secret_service::SecretService::connect(secret_service::EncryptionType::Dh).await
    else {
        return false;
    };

    let collection = match ss.get_default_collection().await {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Check whether there's already an existing item
    let result = collection
        .search_items(HashMap::from([(ATTR_SERVICE, ATTR_VALUE)]))
        .await
        .unwrap_or_default();

    if let Some(item) = result.first() {
        // Update existing item
        if item.is_locked().await.unwrap_or(false) {
            let _ = item.unlock().await;
        }

        return item.set_secret(key.as_bytes(), "text/plain").await.is_ok();
    }

    // Create new item
    collection
        .create_item(
            ITEM_LABEL,
            HashMap::from([(ATTR_SERVICE, ATTR_VALUE)]),
            key.as_bytes(),
            true, // replace if exists
            "text/plain",
        )
        .await
        .is_ok()
}

/// Store the API key. Prefers Secret Service; falls back to GSettings string.
pub async fn store_api_key(key: &str) -> StorageMode {
    let mode = if try_store_in_secret(key).await {
        StorageMode::SecretService
    } else {
        warn!("Secret Service unavailable; storing AI API key in plain GSettings string");
        settings_manager::set_string(Key::AiApiSecret, key.to_string());
        StorageMode::GSettings
    };

    LAST_MODE.set(mode);
    mode
}

/// Read the API key. Tries Secret Service first, then falls back to GSettings.
pub async fn read_api_key() -> Option<String> {
    if let Some(key) = try_read_from_secret().await {
        LAST_MODE.set(StorageMode::SecretService);
        return Some(key);
    };

    // Fall back to GSettings
    let value = settings_manager::string(Key::AiApiSecret);
    if value.is_empty() {
        LAST_MODE.set(StorageMode::GSettings);
        None
    } else {
        LAST_MODE.set(StorageMode::GSettings);
        Some(value)
    }
}

/// Returns the StorageMode used during the most recent read/write operation.
/// Useful for UI feedback (e.g. an info bar explaining the storage situation).
pub fn last_storage_mode() -> StorageMode {
    LAST_MODE.get()
}
