# ADR 0020: Accounts and launch links

- Status: proposed (2026-11-04); the token handoff below waits on a check of the Azure and Minecraft services rules

## Context

Players sign in on the website and press Play on a server page. The client must open, already signed in, and join that server, as the Roblox launch flow does. A Microsoft account that owns Minecraft stays required ([ADR 0007](0007-assets-and-accounts.md)). If the player opens the client directly without having come from the website, the client shows its own Microsoft sign-in.

Joining an online-mode server needs a Minecraft access token for the session join. A Microsoft refresh token is bound to the application and flow that issued it, so a token from the website's sign-in may not be usable by the client's public device-code application. Minecraft services also accept only application registrations that Mojang approved.

## Decision

- **Photon account.** The backend keeps a Photon account per Microsoft account. The Photon account holds friends, cosmetics and settings. The website signs in with Microsoft, and the backend creates or finds the Photon account.
- **Launch link.** Play on the website asks the backend for a one-time launch code (valid 60 s, single use) and opens `photon://launch?code=<code>&server=<id>`. The OS hands the link to the client. If the client is already running, the running instance takes the link. If no client is installed, the page shows the download.
- **Code exchange.** The client sends the code to the backend over TLS. The backend returns the Photon session and the target server. It never puts a token into the link itself.
- **Minecraft token: recommended option (a).** The website's sign-in uses the same Azure application as the client, with a web platform added. The backend keeps the Microsoft refresh token encrypted and returns a short-lived Minecraft access token in the code exchange. The client stores nothing it did not get from its own sign-in.
- **Alternatives kept open:**
  - **(b)** The client signs in once with its own device-code flow and keeps the token in the keyring. After that, the website login only carries the Photon session.
  - **(c)** Photon servers trust a backend ticket instead of Mojang's session server. Vanilla servers would still need (a) or (b).
- **Link registration.** `photon://` is registered per platform: a `.desktop` entry with `x-scheme-handler/photon` on Linux, and registry keys under `HKCU\Software\Classes\photon` on Windows. The installer or first run does this. macOS comes later.

## Open before acceptance

- Check that one Azure application can serve both the public device-code flow and a web redirect flow, and that the Minecraft services approval covers the web flow.
- Decide whether the backend may hold Microsoft refresh tokens (security and privacy review), or fall back to (b).
