//! `orch uninstall` — désinstallation.

use anyhow::Result;

use crate::present;

pub fn run() -> Result<()> {
    present::uninstall()
}