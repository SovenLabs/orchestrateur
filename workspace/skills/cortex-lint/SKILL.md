---
name: cortex-lint
description: Vérifie la cohérence du vault Cortex (orphelins, liens, kinds).
---

# cortex-lint

Checks recommandés sur `workspace/memories/` :

1. Mémoires sans backlinks entrants (orphelins)
2. Wikilinks `[[uuid]]` cassés
3. Kinds manquants ou `context` par défaut abusif
4. Drafts en attente non publiés (`orchestrateur draft list`)

Commandes : `orchestrateur graph`, `orchestrateur draft list`, `orchestrateur audit`.