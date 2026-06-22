use flume::Sender;

/// Événement de streaming émis pendant un tour agent (Phase 8 gateway).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStreamEvent {
    /// Fragment de texte assistant (delta).
    Delta {
        /// Contenu textuel.
        content: String,
    },
    /// Début d'exécution d'un outil.
    ToolStart {
        /// Nom de l'outil.
        name: String,
    },
    /// Fin d'exécution d'un outil.
    ToolEnd {
        /// Nom de l'outil.
        name: String,
        /// Succès de l'exécution.
        success: bool,
    },
    /// Tour terminé.
    End {
        /// Réponse finale assistant.
        reply: String,
        /// Outils invoqués pendant le tour.
        tools_invoked: Vec<String>,
    },
}

/// Sink optionnel pour les événements de streaming agent.
#[derive(Clone, Default)]
pub struct AgentStreamSink {
    tx: Option<Sender<AgentStreamEvent>>,
}

impl AgentStreamSink {
    /// Crée un sink sans émission.
    #[must_use]
    pub fn noop() -> Self {
        Self { tx: None }
    }

    /// Crée un sink branché sur un canal flume.
    #[must_use]
    pub fn from_sender(tx: Sender<AgentStreamEvent>) -> Self {
        Self { tx: Some(tx) }
    }

    /// Émet un événement si un canal est configuré (erreurs ignorées).
    pub fn emit(&self, event: AgentStreamEvent) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(event);
        }
    }
}