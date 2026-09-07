//! Whether macOS actually trusts Kivo to observe the keyboard.
//!
//! This matters because the failure is silent. `CGEventTapCreate` can hand back
//! a tap that never delivers an event, so the listener sits there looking
//! healthy while the user presses the key and nothing happens. Asking the
//! system directly turns "it doesn't work" into a fact.

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
}

/// True when this process may observe and post keyboard events.
///
/// On macOS this is the Accessibility permission in Privacy & Security. It is
/// tied to the app's code signature, so a rebuild invalidates a grant that the
/// System Settings list still shows as enabled — which is its own trap.
pub fn is_trusted() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        AXIsProcessTrusted()
    }
    #[cfg(not(target_os = "macos"))]
    true
}

/// Ask macOS for Accessibility permission, showing the system dialog.
///
/// Must be called on the AppKit main thread. Raised from a worker, the prompt
/// waits for the main run loop while the main run loop waits for the worker,
/// and the process is marked "not responding".
///
/// Worth doing rather than only linking to the settings pane: the prompt puts
/// Kivo into the Accessibility list itself, so the user has something to switch
/// on instead of hunting for an "add" button. Returns the trust state as it
/// stands — granting is asynchronous and needs a restart to take effect,
/// because the permission is read when the event tap is created.
#[cfg(target_os = "macos")]
pub fn request_trust() -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
    let opts =
        CFDictionary::from_CFType_pairs(&[(key.as_CFType(), CFBoolean::true_value().as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(opts.as_concrete_TypeRef() as *const _) }
}

#[cfg(not(target_os = "macos"))]
pub fn request_trust() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_check_runs_without_prompting_or_crashing() {
        // AXIsProcessTrusted must not raise the system dialog — that is
        // AXIsProcessTrustedWithOptions with kAXTrustedCheckOptionPrompt.
        // Value depends on the machine, so only the call is asserted.
        let _ = is_trusted();
    }
}
