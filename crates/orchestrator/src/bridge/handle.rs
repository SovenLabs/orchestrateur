use cortex::DomainEvent;
use flume::{Receiver, Sender};

use super::command::Command;
use super::error::BridgeError;
use super::events::FanoutEventPublisher;
use super::response::Response;

/// Contrat d'inversion de dépendance entre présentation (HUD) et orchestrateur.
///
/// Implémentations : [`ChannelHandle`] (production), mocks de test.
pub trait OrchestratorHandle: Send + Sync + Clone {
    /// Envoie une commande vers le thread orchestrateur (non bloquant).
    ///
    /// # Errors
    ///
    /// Retourne [`BridgeError::ChannelClosed`] si le thread orchestrateur s'est arrêté.
    fn send_command(&self, cmd: Command) -> Result<(), BridgeError>;

    /// Tente de recevoir une réponse sans bloquer le thread UI.
    ///
    /// # Errors
    ///
    /// Retourne [`BridgeError::ChannelClosed`] si le canal est fermé.
    fn try_recv_response(&self) -> Result<Option<Response>, BridgeError>;

    /// Ouvre un abonnement aux événements de domaine (fan-out).
    fn subscribe_events(&self) -> Receiver<DomainEvent>;
}

/// Handle local basé sur des canaux `flume` — utilisé par le HUD egui.
#[derive(Clone)]
pub struct ChannelHandle {
    cmd_tx: Sender<Command>,
    resp_rx: Receiver<Response>,
    events: FanoutEventPublisher,
}

impl ChannelHandle {
    /// Construit un handle depuis les extrémités des canaux (usage interne au runtime).
    #[must_use]
    pub(crate) fn new(
        cmd_tx: Sender<Command>,
        resp_rx: Receiver<Response>,
        events: FanoutEventPublisher,
    ) -> Self {
        Self {
            cmd_tx,
            resp_rx,
            events,
        }
    }
}

impl OrchestratorHandle for ChannelHandle {
    fn send_command(&self, cmd: Command) -> Result<(), BridgeError> {
        self.cmd_tx
            .send(cmd)
            .map_err(|_| BridgeError::ChannelClosed)
    }

    fn try_recv_response(&self) -> Result<Option<Response>, BridgeError> {
        match self.resp_rx.try_recv() {
            Ok(response) => Ok(Some(response)),
            Err(flume::TryRecvError::Empty) => Ok(None),
            Err(flume::TryRecvError::Disconnected) => Err(BridgeError::ChannelClosed),
        }
    }

    fn subscribe_events(&self) -> Receiver<DomainEvent> {
        self.events.subscribe()
    }
}
