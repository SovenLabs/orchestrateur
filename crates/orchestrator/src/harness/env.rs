//! Variables d'environnement harness (token daemon).

use std::process::Command as OsCommand;

use uuid::Uuid;

use crate::harness::error::HarnessError;

/// Nom variable token daemon WS.
pub const DAEMON_TOKEN_ENV: &str = "ORCHESTRATEUR_DAEMON_TOKEN";

/// Définit une variable d'environnement utilisateur persistante (Windows) ou session.
pub fn set_user_env_var(name: &str, value: &str) -> Result<(), HarnessError> {
    #[cfg(windows)]
    {
        let escaped = value.replace('\'', "''");
        let status = OsCommand::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "[Environment]::SetEnvironmentVariable('{name}', '{escaped}', 'User')"
                ),
            ])
            .status()
            .map_err(|e| HarnessError::Platform(e.to_string()))?;
        if !status.success() {
            return Err(HarnessError::Platform(format!(
                "écriture variable utilisateur {name}"
            )));
        }
        std::env::set_var(name, value);
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        std::env::set_var(name, value);
        Ok(())
    }
}

/// Garantit un token daemon (génère si absent).
pub fn ensure_daemon_token() -> Result<bool, HarnessError> {
    if std::env::var(DAEMON_TOKEN_ENV).is_ok() {
        return Ok(false);
    }

    #[cfg(windows)]
    {
        let output = OsCommand::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "[Environment]::GetEnvironmentVariable('{DAEMON_TOKEN_ENV}', 'User')"
                ),
            ])
            .output()
            .map_err(|e| HarnessError::Platform(e.to_string()))?;
        let existing = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !existing.is_empty() {
            std::env::set_var(DAEMON_TOKEN_ENV, &existing);
            return Ok(false);
        }
        let token = Uuid::now_v7().as_simple().to_string();
        set_user_env_var(DAEMON_TOKEN_ENV, &token)?;
        return Ok(true);
    }

    #[cfg(not(windows))]
    {
        let token = Uuid::now_v7().as_simple().to_string();
        std::env::set_var(DAEMON_TOKEN_ENV, &token);
        Ok(true)
    }
}