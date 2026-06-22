# ARCHIVE — Phase 12 v0.12.0

**Date :** Juin 2026  
**Version Cargo :** 0.12.0  
**Tag suggéré :** `phase12-v0.12.0`

---

## Objectif Phase 12

Plugins natifs `.dll`/`.so` (libloading), inbound HTTP pour canaux stub, outils agent `skill_list` / `skill_execute` (agentic).

---

## Livrables

### Plugins natifs (`feature plugins-native`)

| Module | Rôle |
|--------|------|
| `skills/native.rs` | `NativePluginSkill` — ABI FFI + libloading |
| `plugins/pong-native/` | Crate `cdylib` démo |
| `skills/manifest.rs` | `kind = native` + section `[native]` |

**ABI plugin :**

- `orchestrateur_skill_execute(ctx_json: *const c_char) -> *mut c_char`
- `orchestrateur_skill_free(ptr: *mut c_char)` (optionnel)
- Entrée : JSON `SkillContext` · Sortie : `{"message":"..."}` ou `{"error":"..."}`

### Inbound canaux stub (gateway)

| Route | Rôle |
|-------|------|
| `POST /v1/channels/{channel_id}/inbound` | Webhook générique whatsapp, matrix, … |
| Header | `X-Orchestrateur-Channel-Token` |
| Corps | Même format que webhook (`message`, `session_key`, `external_id`) |

`verify_channel_token()` dans `gateway/mod.rs`.

### Outils agent agentic

| Outil | Rôle |
|-------|------|
| `skill_list` | Liste skills builtin / hub / native |
| `skill_execute` | Exécute une skill par nom + contexte JSON |

Toolset `skills` (7 toolsets) + inclus dans `agent` par défaut.  
Config `[agent] skill_tools_enabled = true`.

### Configuration TOML

```toml
[agent]
skill_tools_enabled = true
active_toolset = "agent"
```

---

## Vérification

```powershell
cargo build --release -p orchestrateur-plugin-pong-native
cargo test -p orchestrator --features gateway
cargo test -p orchestrator --features gateway,plugins-native
cargo test -p orchestrator gateway_stub_inbound
cargo test -p orchestrator skills_hub
cargo clippy -p orchestrator --features gateway,plugins-native -- -D warnings
```

**Test inbound stub :**

```powershell
$env:WHATSAPP_TOKEN = "secret"
$env:ORCHESTRATEUR_GATEWAY_TOKEN = "gw"
orchestrateur gateway run
# POST http://127.0.0.1:18789/v1/channels/whatsapp/inbound
# Header: X-Orchestrateur-Channel-Token: secret
# Body: {"message":"Hello stub","session_key":"whatsapp:1"}
```

---

## Hors scope (Phase 13+)

- Marketplace skills distant
- Polling réel WhatsApp / Matrix SDK
- Agent auto-sélection skill sans tool call explicite
- Signature / notarisation plugins natifs

---

*Orchestrateur — Sovën, 2026*