//! Plugin natif démo Phase 12 — ABI Orchestrateur.
//!
//! Exports :
//! - `orchestrateur_skill_execute(ctx_json: *const c_char) -> *mut c_char`
//! - `orchestrateur_skill_free(ptr: *mut c_char)`

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Exécute la skill — entrée/sortie JSON.
///
/// # Safety
///
/// `ctx` doit être un pointeur valide vers une chaîne UTF-8 nulle-terminée.
#[no_mangle]
pub unsafe extern "C" fn orchestrateur_skill_execute(ctx: *const c_char) -> *mut c_char {
    let _input = if ctx.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ctx).to_string_lossy().into_owned()
    };
    let response = serde_json::json!({ "message": "pong-native" });
    CString::new(response.to_string())
        .unwrap_or_else(|_| CString::new(r#"{"message":"pong-native"}"#).expect("fallback"))
        .into_raw()
}

/// Libère une chaîne allouée par le plugin.
///
/// # Safety
///
/// `ptr` doit provenir de [`orchestrateur_skill_execute`] et ne pas être libéré deux fois.
#[no_mangle]
pub unsafe extern "C" fn orchestrateur_skill_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}