# Schémas skills — hub, marketplace et contrats (P6)

**Version document :** 1.0 · **Orchestrateur :** 0.28.0 · **Schémas :** hub `1.0`, catalogue `1`

Document canonique pour les extensions Esprit (P6). Le noyau Rust (P0–P5) expose des **contrats stables** ; les skills évoluent dans `workspace/skills/`, `plugins/` et le catalogue marketplace sans modifier le code orchestrateur.

Voir aussi : [`project-hierarchy.md`](project-hierarchy.md) · [`hermes-tools.md`](hermes-tools.md) · [`workspace/skills/RESOLVER.md`](../workspace/skills/RESOLVER.md) · [`harness-integral.md`](harness-integral.md)

---

## 1. Trois types de skills

| Type | Emplacement | Exécution | Évolutif post-gel |
|------|-------------|-----------|-------------------|
| **Built-in** | `crates/orchestrator/src/skills/` | Trait `Skill` compilé | Non (gelé avec P2) |
| **Hub plugin** | `workspace/skills/<id>/skill.toml` | Subprocess ou DLL native | Oui |
| **Markdown prompt** | `workspace/skills/<id>/SKILL.md` | Routage agent / IDE | Oui |

**Règle absolue :** aucune skill n'écrit directement dans le vault mémoire. Toute persistance passe par les outils Cortex (`assimilate`, `search`, `draft_publish`, MCP `cortex_*`).

---

## 2. Schéma hub — `skill.toml` (v1.0)

Fichier TOML à la racine d'un répertoire skill : `workspace/skills/<id>/skill.toml`.

### Sections

#### `[skill]` (requis)

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `id` | string | oui | Identifiant stable (= nom du répertoire recommandé) |
| `name` | string | non | Nom affiché (défaut : `id`) |
| `description` | string | non | Description lisible |
| `version` | string | non | Semver libre (défaut : `0.1.0`) |
| `kind` | enum | non | `subprocess` (défaut) ou `native` |
| `enabled` | bool | non | Active le chargement (défaut : `true`) |
| `integrity_hash` | string | non | Empreinte BLAKE3 du manifeste (voir §4) |

#### `[subprocess]` (si `kind = subprocess`)

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `command` | string | oui | Exécutable |
| `args` | string[] | non | Arguments |
| `stdin_json` | bool | non | Envoie `SkillContext` en JSON sur stdin (défaut : `false`) |
| `timeout_secs` | u64 | non | Timeout (défaut : `30`) |

#### `[native]` (si `kind = native`)

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `library` | string | oui | Chemin `.dll` / `.so` (relatif au répertoire skill ou absolu) |

### Exemple subprocess

```toml
[skill]
id = "pong"
name = "Pong"
description = "Plugin démo — retourne pong"
version = "0.1.0"
enabled = true

[subprocess]
command = "cmd"
args = ["/c", "echo", "pong"]
stdin_json = false
timeout_secs = 5
```

### Exemple native

```toml
[skill]
id = "pong-native"
kind = "native"

[native]
library = "pong_native.dll"
```

Implémentation de référence : [`crates/orchestrator/src/skills/manifest.rs`](../crates/orchestrator/src/skills/manifest.rs).

---

## 3. Schéma marketplace — `catalog.json` (v1)

Fichier JSON référencé par `orchestrator.toml` :

```toml
[skills_hub]
marketplace_enabled = true
marketplace_catalog = "skills/marketplace/catalog.json"
# marketplace_url = "https://example.com/skills/catalog.json"
# marketplace_require_signature = false
```

### Racine

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `version` | u32 | oui | **Doit être `1`** pour ce schéma |
| `catalog_hash` | string | non | Empreinte BLAKE3 du catalogue sans ce champ (voir §4) |
| `skills` | array | oui | Entrées installables |

### Entrée `skills[]`

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `id` | string | oui | Répertoire cible `workspace/skills/<id>/` |
| `name` | string | oui | Nom affiché |
| `description` | string | oui | Description catalogue |
| `version` | string | oui | Version catalogue de l'entrée |
| `enabled` | bool | non | Installe lors d'un `sync` (défaut : `true`) |
| `manifest_toml` | string | oui | Contenu complet du `skill.toml` à écrire |

### Exemple minimal

```json
{
  "version": 1,
  "skills": [
    {
      "id": "market-echo",
      "name": "Market Echo",
      "description": "Echo installé depuis le catalogue",
      "version": "0.1.0",
      "enabled": true,
      "manifest_toml": "[skill]\nid = \"market-echo\"\n..."
    }
  ]
}
```

Catalogue de dev : [`workspace/skills/marketplace/catalog.json`](../workspace/skills/marketplace/catalog.json).

---

## 4. Empreintes BLAKE3

### `integrity_hash` (manifeste hub)

1. Retirer toute ligne commençant par `integrity_hash` ou `integrity-hash`.
2. Concaténer les lignes restantes avec `\n`.
3. BLAKE3 du texte canonique → hex (préfixe `blake3:` accepté à la vérification).

```powershell
# Générer (via orchestrator)
cargo test -p orchestrator manifest::tests::integrity_hash_roundtrip -- --nocapture
```

À la sync marketplace, le noyau calcule et injecte `integrity_hash` dans le `skill.toml` écrit.

### `catalog_hash` (catalogue)

1. Cloner le catalogue JSON sans le champ `catalog_hash`.
2. Sérialiser en JSON compact (ordre des champs stable via `serde`).
3. BLAKE3 → hex.

Si `marketplace_require_signature = true` dans `orchestrator.toml`, `catalog_hash` est **obligatoire**.

---

## 5. Configuration `orchestrator.toml` — `[skills_hub]`

| Champ | Type | Défaut | Description |
|-------|------|--------|-------------|
| `enabled` | bool | `true` | Active le hub |
| `directory` | string | `"skills"` | Relatif au workspace |
| `auto_load` | bool | `true` | Charge les plugins au démarrage |
| `marketplace_enabled` | bool | `true` | Active le catalogue |
| `marketplace_catalog` | string | `"skills/marketplace/catalog.json"` | Chemin relatif |
| `marketplace_url` | string | — | URL distante (feature `skills-marketplace`) |
| `marketplace_require_signature` | bool | `false` | Exige `catalog_hash` |

### Entrées inline `[[skills_hub.entries]]`

Alternative aux fichiers : skill subprocess déclarée directement dans la config (sans `skill.toml`).

```toml
[[skills_hub.entries]]
id = "inline-echo"
description = "Echo inline depuis TOML"
command = "echo"
args = ["inline"]
enabled = false
stdin_json = false
timeout_secs = 10
```

---

## 6. Contrat trait `Skill` (noyau figé)

```rust
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn source(&self) -> SkillSource;  // Builtin | Hub | Native
    fn version(&self) -> Option<&str> { None }
    async fn execute(&self, ctx: SkillContext) -> Result<SkillOutput, SkillError>;
}
```

### `SkillContext` (entrée bridge / CLI)

| Champ | Usage |
|-------|-------|
| `query` | Recherche sémantique (`search`) |
| `text` | Texte à assimiler (`assimilate`) |
| `tags` | Filtres ou contexte |
| `limit` | Limite de résultats |

### Built-in gelées (P2)

| Nom | Rôle |
|-----|------|
| `noop` | Test / placeholder |
| `list_memories` | Liste les mémoires Cortex |
| `search` | Recherche sémantique |
| `assimilate` | Crée une mémoire via LLM |

---

## 7. Skills Markdown (`SKILL.md`)

Fichiers de routage pour agents et IDE — **pas** des plugins exécutables.

```markdown
---
name: cortex-capture
description: Capture une idée dans le Cortex via assimilation structurée.
---
```

Table de routage : [`workspace/skills/RESOLVER.md`](../workspace/skills/RESOLVER.md).

---

## 8. Plugins natifs Rust (`plugins/`)

Crate Rust compilée en bibliothèque dynamique, référencée par `kind = "native"`.

Référence : [`plugins/pong-native/`](../plugins/pong-native/).

---

## 9. CLI harness

```powershell
orch skill list                              # built-in + hub chargées
orch skill run <name> [--query] [--text] [--tags] [--limit]
orch skill install <id>                      # depuis le catalogue
orch skill update                            # sync catalogue → hub

# Alias legacy (cachés)
orch skills-hub list
orch skills-hub sync
orch skills-hub verify
```

---

## 10. Politique PR post-gel (skills-only)

Après tag semver de gel du noyau (P0–P5) :

| Zone | PR acceptées |
|------|--------------|
| `workspace/skills/**` | Oui — nouvelles skills, catalogue, SKILL.md |
| `plugins/**` | Oui — plugins natifs |
| `docs/skills*.md`, `RESOLVER.md` | Oui — documentation skills |
| `crates/**`, `apps/**` | **Non** — sauf correctif sécurité critique |

Les skills ne doivent **pas** importer de modules internes `orchestrator::` hors API publique documentée (trait `Skill`, bridge `Command`/`Response`, outils MCP).

---

## 11. Évolution des schémas

| Schéma | Version actuelle | Règle de compatibilité |
|--------|------------------|------------------------|
| `skill.toml` | 1.0 | Ajout de champs optionnels autorisé ; champs requis inchangés |
| `catalog.json` | `version: 1` | Incrémenter `version` pour tout changement breaking |

Prochaine version catalogue : `version: 2` avec section de migration explicite dans ce document.