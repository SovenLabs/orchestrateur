//! `orch uninstall` — désinstallation.

use anyhow::Result;

use crate::harness_ops::cmd_uninstall;

pub fn run() -> Result<()> {
    cmd_uninstall()
}