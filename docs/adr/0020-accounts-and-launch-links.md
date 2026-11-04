# ADR 0020: Accounts and launch links

- Status: accepted (2026-11-04); the silent sign-in detail ("Minecraft token") is verified in its own slice

## Context

Players sign in on the website and press Play on a game page. The client must open, already signed in, and join that server, as the Roblox launch flow does. A Microsoft account that owns Minecraft stays required ([ADR 0007](0007-assets-and-accounts.md)). If the player opens the client directly, the client shows its own Microsoft sign-in.

The backend must not keep users' Microsoft tokens: that is a liability, and players would rightly distrust it. A raw token in the `photon://` link is no answer either:

- Other local processes can read a link from the handler's command line (`/proc/<pid>/cmdline` on Linux, the process list on Windows), and it can end up in browser history and logs.
- Any web page can open a `photon://` link. A link that carries a token would let a hostile page sign the client into an attacker's account.
- The website would still need an approved application to obtain a Minecraft token in the first place.

## Decision

- **Photon account.** The backend keeps a Photon account per Minecraft profile UUID. It holds friends, cosmetics and settings. It stores no Microsoft or Minecraft tokens, only the Photon session.
- **The link carries an opaque code.** Play on a game page asks the backend for a launch code (single use, valid 60 s) and opens `photon://launch?code=<code>`. The code names the Photon session and the directory id of the game server. It is not a Microsoft or Minecraft token. If no client is installed, the page shows the download.
- **Server is a directory id.** The backend resolves the address after the code exchange. A link can never name a host, so a crafted link cannot send the client to an arbitrary address.
- **Code exchange.** The client sends the code to the backend over TLS and gets the game server's address and the UUID the code is bound to.
- **UUID binding.** The client compares that UUID with its signed-in profile. If they differ, it asks before switching accounts and never joins silently as someone else.
- **The Minecraft token comes from the client's own sign-in.** It signs in once and keeps the refresh token in the OS keyring (works today). To make this feel automatic, the client tries an authorization-code flow with PKCE through the system browser and a `localhost` redirect: the player is usually already signed in to Microsoft there, so it is one click. Device code stays as the fallback. Whether the Azure application allows this redirect is checked in its platform settings before the slice is built.
- **Link registration.** `photon://` is registered per platform: a `.desktop` entry with `x-scheme-handler/photon` on Linux, and keys under `HKCU\Software\Classes\photon` on Windows. A running client takes the link (single instance). macOS comes later.

## Consequences

- No Microsoft token ever leaves the player's machine.
- The "auto sign-in" is one browser click, not zero, until Microsoft allows more for this application.
- Photon servers may later trust a backend ticket instead of the session server; vanilla servers keep using the Mojang session join.
