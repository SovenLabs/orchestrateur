---
name: esprit-review
description: Revue périodique Esprit + gouvernance drafts watcher.
---

# esprit-review

Boucle hebdomadaire harness intégré :

1. `orchestrateur watcher status` — sessions traitées
2. `orchestrateur draft list` — publier ou rejeter chaque brouillon
3. `orchestrateur graph` — hubs et densité du graphe
4. Tour `esprit_chat` : « Quels patterns émergent cette semaine ? »

Toute décision durable → `draft_publish` ou `assimilate` avec kind `decision`.