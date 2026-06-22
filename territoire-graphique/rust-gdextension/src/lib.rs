use godot::prelude::*;
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};

struct OrchestrateurGodotExtension;

#[gdextension]
unsafe impl ExtensionLibrary for OrchestrateurGodotExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct BrainBridge {
    base: Base<RefCounted>,
    runtime: Arc<Mutex<Option<Runtime>>>,
}

#[godot_api]
impl IRefCounted for BrainBridge {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            base,
            runtime: Arc::new(Mutex::new(None)),
        }
    }
}

#[godot_api]
impl BrainBridge {
    #[func]
    fn get_brain_activity(&self) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.insert("intensity", 0.85);
        dict.insert("pulse_speed", 1.4);
        dict.insert("particle_count", 65);
        dict
    }

    #[func]
    fn start_websocket_server(&self) {
        // TODO: Lancer un serveur WebSocket sur le port 28790
        // Pour l'instant en simulation
        godot_print!("WebSocket server would start on port 28790");
    }
}