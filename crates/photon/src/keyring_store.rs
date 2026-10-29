//! The operating system's credential store as the place a signed-in player's refresh token is
//! kept (ADR 0007): never a file this client writes. Linux uses the Secret Service, macOS the
//! Keychain and Windows the Credential Manager.

use photon_net::account::AccessToken;
use photon_net::signin::{SecretStore, SignInError};

/// The service name entries are kept under.
const SERVICE: &str = "photon-client";
/// The one entry: the refresh token of the player signed in.
const ENTRY: &str = "microsoft-refresh-token";

/// The credential store of this machine, ready to use.
pub(crate) fn os_store() -> Box<dyn SecretStore> {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        Box::new(native::NativeStore::connect())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Box::new(Unsupported)
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
struct Unsupported;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
impl SecretStore for Unsupported {
    fn load(&self) -> Result<Option<AccessToken>, SignInError> {
        Ok(None)
    }

    fn save(&self, _: &AccessToken) -> Result<(), SignInError> {
        Err(SignInError::Store(
            "no credential store on this platform yet".to_owned(),
        ))
    }

    fn clear(&self) -> Result<(), SignInError> {
        Ok(())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod native {
    use super::*;
    use keyring_core::{Entry, Error};

    /// Makes this platform's store the default one: the freedesktop Secret Service (GNOME
    /// Keyring, KWallet, ...), the macOS Keychain or the Windows Credential Manager.
    fn open_store() -> Result<(), String> {
        #[cfg(target_os = "linux")]
        let store = dbus_secret_service_keyring_store::Store::new();
        #[cfg(target_os = "macos")]
        let store = apple_native_keyring_store::keychain::Store::new();
        #[cfg(target_os = "windows")]
        let store = windows_native_keyring_store::Store::new();
        keyring_core::set_default_store(store.map_err(|error| error.to_string())?);
        Ok(())
    }

    /// The operating system's credential store, or the reason it is not there.
    pub(super) struct NativeStore {
        entry: Result<Entry, String>,
    }

    impl NativeStore {
        pub(super) fn connect() -> Self {
            let entry = open_store()
                .and_then(|()| Entry::new(SERVICE, ENTRY).map_err(|error| error.to_string()));
            Self { entry }
        }

        fn entry(&self) -> Result<&Entry, SignInError> {
            self.entry
                .as_ref()
                .map_err(|reason| SignInError::Store(reason.clone()))
        }
    }

    fn failed(error: &Error) -> SignInError {
        SignInError::Store(error.to_string())
    }

    impl SecretStore for NativeStore {
        fn load(&self) -> Result<Option<AccessToken>, SignInError> {
            match self.entry()?.get_password() {
                Ok(token) => Ok(Some(AccessToken::new(token))),
                Err(Error::NoEntry) => Ok(None),
                Err(error) => Err(failed(&error)),
            }
        }

        fn save(&self, token: &AccessToken) -> Result<(), SignInError> {
            self.entry()?
                .set_password(token.reveal_for_storage())
                .map_err(|error| failed(&error))
        }

        fn clear(&self) -> Result<(), SignInError> {
            match self.entry()?.delete_credential() {
                Ok(()) | Err(Error::NoEntry) => Ok(()),
                Err(error) => Err(failed(&error)),
            }
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    /// Needs a running Secret Service (a desktop session); run with `--ignored`. It uses the
    /// real entry, so it saves the player's own token first and puts it back afterwards.
    #[test]
    #[ignore = "needs a Secret Service and touches the real entry"]
    fn the_secret_service_keeps_and_forgets_a_token() {
        let store = os_store();
        let before = store.load().expect("the service answers");
        store
            .save(&AccessToken::new("probe".to_owned()))
            .expect("saved");
        let loaded = store.load().expect("loaded").expect("something is kept");
        assert_eq!(loaded.reveal_for_storage(), "probe");
        store.clear().expect("cleared");
        assert!(store.load().expect("loaded").is_none());
        if let Some(token) = before {
            store.save(&token).expect("restored");
        }
    }
}
