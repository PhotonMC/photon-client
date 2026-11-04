//! `photon identity-proof <server-id>`: joins a session server id as the signed-in player, the
//! way the backend's identity challenge asks (ADR 0022). A development tool: it prints the name
//! to ask `hasJoined` about, never the token.

use std::time::Duration;

use photon_net::connection::prove_identity;

/// Joins `server_id` at the session host with the stored sign-in and says whose name to check.
pub(crate) fn run(server_id: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let server_id = server_id.ok_or("usage: photon identity-proof <server-id>")?;
    let (name, account) = crate::profile_login::identity()?;
    let account = account.ok_or("nobody is signed in: run `photon sign-in`")?;
    prove_identity(&account, &server_id, Duration::from_secs(10))?;
    println!("joined {server_id} as {name}");
    Ok(())
}
