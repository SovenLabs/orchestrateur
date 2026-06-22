use std::time::Duration;

use flume::Receiver;
use orchestrator::{
    bridge::{BridgeError, Command, OrchestratorHandle, Response},
    spawn_orchestrator_bridge, AppDependencies, ChannelHandle, DomainEvent, OrchestratorThread,
};
use thiserror::Error;
use tokio::time::timeout;

/// Erreurs du client Bridge haut niveau.
#[derive(Debug, Error)]
pub enum ClientError {
    /// Échec de communication avec le thread orchestrateur.
    #[error("bridge: {0}")]
    Bridge(#[from] BridgeError),
    /// Délai dépassé en attente d'une réponse.
    #[error("timeout après {0:?}")]
    Timeout(Duration),
}

/// Client haut niveau : handle clonable + thread orchestrateur joignable.
pub struct OrchestratorClient {
    handle: ChannelHandle,
    thread: Option<OrchestratorThread>,
}

impl OrchestratorClient {
    /// Démarre le thread Bridge + [`OrchestratorFacade`] et retourne un client prêt.
    ///
    /// # Errors
    ///
    /// Propage [`BridgeError`] si le thread ne peut pas démarrer.
    pub fn connect(deps: AppDependencies) -> Result<Self, BridgeError> {
        let (handle, thread) = spawn_orchestrator_bridge(deps)?;
        Ok(Self {
            handle,
            thread: Some(thread),
        })
    }

    /// Handle clonable pour envoi/polling (clients bridge embarqués).
    #[must_use]
    pub fn handle(&self) -> &ChannelHandle {
        &self.handle
    }

    /// Canal d'événements domaine (fan-out).
    #[must_use]
    pub fn subscribe_events(&self) -> Receiver<DomainEvent> {
        self.handle.subscribe_events()
    }

    /// Extrait le thread orchestrateur (usage unique à l'arrêt du client).
    pub fn take_thread(&mut self) -> Option<OrchestratorThread> {
        self.thread.take()
    }

    /// Envoie une commande et attend la réponse correspondante (avec timeout).
    ///
    /// # Errors
    ///
    /// Retourne [`ClientError::Bridge`] ou [`ClientError::Timeout`].
    pub async fn execute(
        &self,
        cmd: Command,
        wait: Duration,
    ) -> Result<Response, ClientError> {
        self.handle.send_command(cmd)?;
        timeout(wait, Self::recv_one(&self.handle))
            .await
            .map_err(|_| ClientError::Timeout(wait))?
    }

    async fn recv_one(handle: &ChannelHandle) -> Result<Response, ClientError> {
        loop {
            if let Some(response) = handle.try_recv_response()? {
                return Ok(response);
            }
            tokio::task::yield_now().await;
        }
    }
}