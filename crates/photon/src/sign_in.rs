//! `photon sign-in` and `photon sign-out`: the Microsoft account the client plays online with,
//! until the sign-in screen of the first-run flow exists.

use photon_net::signin::{self, ClientId, Hosts};

/// Sign in with a device code shown in the terminal, and keep the refresh token in the
/// operating system's credential store.
pub(crate) fn run_sign_in() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientId::from_environment();
    let hosts = Hosts::default();
    let code = signin::request_device_code(&hosts, &client)?;
    println!(
        "Open {} and enter the code {}",
        code.verification_uri, code.user_code
    );
    let tokens = signin::poll_for_token(&hosts, &client, &code, &mut std::thread::sleep)?;
    let host = crate::profile_login::session_host();
    let store = crate::keyring_store::os_store();
    let session = signin::finish_sign_in(&hosts, &tokens, (&host, store.as_ref()))?;
    println!("Signed in as {}", session.account.name);
    Ok(())
}

/// Forget the signed-in player.
pub(crate) fn run_sign_out() -> Result<(), Box<dyn std::error::Error>> {
    signin::sign_out(crate::keyring_store::os_store().as_ref())?;
    println!("Signed out");
    Ok(())
}
