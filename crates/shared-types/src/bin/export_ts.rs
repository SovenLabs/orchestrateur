//! Génère les bindings TypeScript pour le frontend Tauri.
//!
//! ```bash
//! cargo run -p shared-types --bin export-ts
//! ```

use std::path::Path;

use ts_rs::TS;
use shared_types::events::{BackendEvent, FrontendCommand};
use shared_types::protocol::{
    ClientInfo, DaemonClientMessage, DaemonServerMessage, TerritoryBroadcast,
};

fn main() {
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/tauri-desktop/src/lib/generated");

    if let Err(err) = std::fs::create_dir_all(&out_dir) {
        eprintln!("impossible de créer {out_dir:?}: {err}");
        std::process::exit(1);
    }

    let cfg = ts_rs::Config::default();
    for export in [
        BackendEvent::export(&cfg),
        FrontendCommand::export(&cfg),
        ClientInfo::export(&cfg),
        TerritoryBroadcast::export(&cfg),
        DaemonClientMessage::export(&cfg),
        DaemonServerMessage::export(&cfg),
        shared_types::ConnectionConfig::export(&cfg),
    ] {
        if let Err(err) = export {
            eprintln!("export TS échoué: {err}");
            std::process::exit(1);
        }
    }

    let index = out_dir.join("index.ts");
    let index_body = r"// Auto-généré par `cargo run -p shared-types --bin export-ts` — ne pas éditer.
export * from './BackendEvent';
export * from './FrontendCommand';
export * from './ClientInfo';
export * from './TerritoryBroadcast';
export * from './DaemonClientMessage';
export * from './DaemonServerMessage';
export * from './ConnectionConfig';
";
    if let Err(err) = std::fs::write(&index, index_body) {
        eprintln!("écriture index.ts échouée: {err}");
        std::process::exit(1);
    }

    println!("Types TS exportés vers {}", out_dir.display());
}