//! Harness — logique métier partagée CLI, desktop, tests.

mod client;
mod daemon;
mod doctor;
mod env;
mod error;
mod onboard;
mod probe;
mod providers;
mod smoke;
mod supervisor;
mod types;
mod workspace;

#[cfg(feature = "gateway")]
mod channels;

pub use client::probe_client;
pub use daemon::{
    install_scheduled_task, probe_daemon_status, probe_gateway_status, scheduled_task_installed,
    service_badges, stop_daemon,
};
pub use doctor::run_doctor;
pub use env::{ensure_daemon_token, set_user_env_var, DAEMON_TOKEN_ENV};
pub use error::HarnessError;
pub use onboard::{run_configure, run_onboard};
pub use probe::{probe_health, probe_harness_services, HarnessServiceProbe, ServiceHealth};
pub use providers::{probe_providers, validate_probe};
pub use smoke::{run_smoke, SmokeResult};
pub use supervisor::{plan_supervisor, spawn_child, wait_for_spawn, SupervisorSpawnPlan};
pub use types::{
    ChannelStatusRow, CheckStatus, ConfigureOptions, DaemonInstallResult, DaemonStopResult,
    DoctorCheck, DoctorReport, HarnessSmokeOptions, OnboardOptions, OnboardResult,
    ProviderProbeResult, ServiceProbeState, ServiceStatusDetail,
};
pub use workspace::{config_path, ensure_workspace_tree, find_example_config, write_minimal_config};

#[cfg(feature = "gateway")]
pub use channels::{disable_channel, enable_channel, list_channel_status};