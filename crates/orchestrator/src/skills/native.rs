use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use libloading::{Library, Symbol};
use thiserror::Error;

use crate::error::SkillError;
use crate::skills::manifest::{NativePluginConfig, SkillManifest, SkillPluginConfig};
use crate::skills::skill::{Skill, SkillContext, SkillOutput, SkillSource};

type ExecuteFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type FreeFn = unsafe extern "C" fn(*mut c_char);

/// Erreurs de chargement plugin natif.
#[derive(Debug, Error)]
pub enum NativePluginError {
    /// Chargement bibliothèque.
    #[error("chargement bibliothèque {path}: {message}")]
    Load {
        /// Chemin tenté.
        path: PathBuf,
        /// Détail.
        message: String,
    },
    /// Symbole ABI manquant.
    #[error("symbole manquant dans {path}: {symbol}")]
    MissingSymbol {
        /// Chemin bibliothèque.
        path: PathBuf,
        /// Nom symbole.
        symbol: String,
    },
    /// Exécution FFI.
    #[error("exécution native: {0}")]
    Execute(String),
}

/// Skill chargée depuis une bibliothèque dynamique (`.dll` / `.so`).
pub struct NativePluginSkill {
    id: String,
    description: String,
    version: String,
    library_path: PathBuf,
    library: Arc<Library>,
}

impl NativePluginSkill {
    /// Charge un plugin natif depuis un manifeste hub.
    ///
    /// # Errors
    ///
    /// Propage [`NativePluginError`] si la bibliothèque ou les symboles ABI sont absents.
    pub fn from_manifest(manifest: SkillManifest) -> Result<Self, NativePluginError> {
        let NativePluginConfig { library } = match manifest.plugin {
            SkillPluginConfig::Native(cfg) => cfg,
            _ => {
                return Err(NativePluginError::Execute(
                    "manifeste non natif".into(),
                ));
            }
        };
        let path = resolve_library_path(&manifest.root, &library);
        Self::load(manifest.id, manifest.description, manifest.version, path)
    }

    fn load(
        id: String,
        description: String,
        version: String,
        path: PathBuf,
    ) -> Result<Self, NativePluginError> {
        let library = Arc::new(load_library(&path).map_err(|e| NativePluginError::Load {
                path: path.clone(),
                message: e.to_string(),
            })?);
        let execute: Symbol<ExecuteFn> =
            lookup_symbol(&library, b"orchestrateur_skill_execute\0").map_err(|_| {
                NativePluginError::MissingSymbol {
                    path: path.clone(),
                    symbol: "orchestrateur_skill_execute".into(),
                }
            })?;
        drop(execute);
        Ok(Self {
            id,
            description,
            version,
            library_path: path,
            library,
        })
    }

    fn lookup_execute(&self) -> Result<Symbol<'_, ExecuteFn>, NativePluginError> {
        lookup_symbol(&self.library, b"orchestrateur_skill_execute\0").map_err(|_| {
            NativePluginError::MissingSymbol {
                path: self.library_path.clone(),
                symbol: "orchestrateur_skill_execute".into(),
            }
        })
    }

    fn lookup_free(&self) -> Option<Symbol<'_, FreeFn>> {
        lookup_symbol(&self.library, b"orchestrateur_skill_free\0").ok()
    }
}

#[allow(unsafe_code)]
fn load_library(path: &Path) -> Result<Library, libloading::Error> {
    unsafe { Library::new(path) }
}

#[allow(unsafe_code)]
fn lookup_symbol<'lib, T>(library: &'lib Library, name: &[u8]) -> Result<Symbol<'lib, T>, libloading::Error> {
    unsafe { library.get(name) }
}

#[async_trait]
impl Skill for NativePluginSkill {
    fn name(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> SkillSource {
        SkillSource::Native
    }

    fn version(&self) -> Option<&str> {
        Some(&self.version)
    }

    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        let execute = self
            .lookup_execute()
            .map_err(|e| SkillError::ExecutionFailed(e.to_string()))?;
        let payload = serde_json::to_string(ctx)
            .map_err(|e| SkillError::ExecutionFailed(format!("json ctx: {e}")))?;
        let c_input = CString::new(payload)
            .map_err(|e| SkillError::ExecutionFailed(format!("cstring ctx: {e}")))?;
        let output_ptr = invoke_execute(execute, c_input.as_ptr());
        if output_ptr.is_null() {
            return Err(SkillError::ExecutionFailed(
                "plugin natif a retourné null".into(),
            ));
        }
        let output = read_c_string(output_ptr);
        if let Some(free) = self.lookup_free() {
            invoke_free(free, output_ptr);
        }
        let parsed: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| SkillError::ExecutionFailed(format!("json stdout: {e}")))?;
        if let Some(err) = parsed.get("error").and_then(|v| v.as_str()) {
            return Err(SkillError::ExecutionFailed(err.to_string()));
        }
        let message = parsed
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                SkillError::ExecutionFailed("stdout JSON doit contenir message ou error".into())
            })?;
        Ok(SkillOutput { message })
    }
}

#[allow(unsafe_code)]
fn invoke_execute(execute: Symbol<'_, ExecuteFn>, ctx: *const c_char) -> *mut c_char {
    unsafe { execute(ctx) }
}

#[allow(unsafe_code)]
fn invoke_free(free: Symbol<'_, FreeFn>, ptr: *mut c_char) {
    unsafe { free(ptr) };
}

#[allow(unsafe_code)]
fn read_c_string(ptr: *mut c_char) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
}

fn resolve_library_path(root: &Path, library: &str) -> PathBuf {
    let candidate = root.join(library);
    if candidate.exists() {
        return candidate;
    }
    PathBuf::from(library)
}