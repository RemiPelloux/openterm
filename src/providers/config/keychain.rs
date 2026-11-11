//! macOS Keychain storage for provider API keys: one generic-password item
//! per provider under a single service. Unit tests and `OPENTERM_KEYCHAIN=off`
//! never touch the login keychain; other platforms have no keychain backend.

#[cfg(target_os = "macos")]
pub const SERVICE: &str = "ai.openterm.providers";
const SWITCH_ENV: &str = "OPENTERM_KEYCHAIN";
const SWITCH_OFF: &str = "off";

/// Secret storage keyed by provider name; injectable so tests use memory.
pub trait KeyStore {
    fn get(&self, account: &str) -> Option<String>;
    fn set(&self, account: &str, secret: &str) -> Result<(), String>;
}

/// The login keychain, when this build and environment allow it.
pub struct Keychain;

impl Keychain {
    pub fn enabled() -> bool {
        cfg!(all(target_os = "macos", not(test)))
            && std::env::var(SWITCH_ENV).map_or(true, |value| value != SWITCH_OFF)
    }
}

impl KeyStore for Keychain {
    fn get(&self, account: &str) -> Option<String> {
        if !Self::enabled() {
            return None;
        }
        backend::get(account)
    }

    fn set(&self, account: &str, secret: &str) -> Result<(), String> {
        if !Self::enabled() {
            return Err("keychain is disabled or unavailable".to_owned());
        }
        backend::set(account, secret)
    }
}

#[cfg(target_os = "macos")]
mod backend {
    use super::SERVICE;

    fn entry(account: &str) -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICE, account).map_err(|error| error.to_string())
    }

    pub fn get(account: &str) -> Option<String> {
        let secret = entry(account).ok()?.get_password().ok()?;
        (!secret.is_empty()).then_some(secret)
    }

    pub fn set(account: &str, secret: &str) -> Result<(), String> {
        entry(account)?
            .set_password(secret)
            .map_err(|error| error.to_string())
    }
}

#[cfg(not(target_os = "macos"))]
mod backend {
    pub fn get(_account: &str) -> Option<String> {
        None
    }

    pub fn set(_account: &str, _secret: &str) -> Result<(), String> {
        Err("no keychain on this platform".to_owned())
    }
}

#[cfg(test)]
pub mod memory {
    use super::KeyStore;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    /// In-memory store; `failing` simulates a locked or missing keychain.
    #[derive(Default)]
    pub struct MemoryStore {
        pub items: RefCell<BTreeMap<String, String>>,
        pub failing: bool,
    }

    impl KeyStore for MemoryStore {
        fn get(&self, account: &str) -> Option<String> {
            self.items.borrow().get(account).cloned()
        }

        fn set(&self, account: &str, secret: &str) -> Result<(), String> {
            if self.failing {
                return Err("locked".to_owned());
            }
            self.items
                .borrow_mut()
                .insert(account.to_owned(), secret.to_owned());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyStore, Keychain};

    #[test]
    fn unit_tests_never_reach_the_login_keychain() {
        assert!(!Keychain::enabled());
        assert_eq!(Keychain.get("groq"), None);
        assert!(Keychain.set("groq", "secret").is_err());
    }
}
