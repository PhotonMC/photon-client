# ADR 0021: Client UI technology and design tokens

- Status: accepted (2026-11-04); the browser engine choice is confirmed by a spike

## Context

The platform ([ADR 0018](0018-photon-platform.md)) has game pages, like Roblox's: a title, a description, a trailer and screenshots, videos (including YouTube), the player count and a Play button. The website shows them, and the client shows the same page before a join. YouTube allows playback only through its official embedded player, which needs a browser engine and a real `https` origin. Downloading YouTube video streams is not an option.

The website uses light and dark skins, and the client uses the game-like "Classic" skin. Both must look like one design.

## Decision

- **Native chrome.** The sidebar, home grid, loading screen, in-game menu and settings are drawn natively on `photon-gui` from design tokens. The client starts, shows the home screen and joins a server without any browser engine.
- **Web surface for game pages.** The body of a game page is the website's own page, rendered by an optional off-screen browser surface into a texture of the Vulkan renderer. This keeps rich layout, video and YouTube out of the native UI kit, and shares one page with the site.
  - **Engine.** An off-screen Chromium Embedded Framework build. System webviews do not render cleanly into our texture, and Servo cannot play YouTube. The spike must confirm that a stock build plays what YouTube serves (VP9 or AV1 with Opus) without proprietary codecs.
  - **Origin.** Game pages load from the photon-web domain over `https`, never from `file://` or a custom scheme, because the YouTube player refuses those.
  - **Delivery.** The engine is downloaded on demand, signed and verified on the update path of [ADR 0015](0015-signed-updates.md), and only when a page needs it. Without it, the native chrome shows the page's title, description and images from the directory data and a link that opens the browser.
- **One token file.** Design tokens (colors per skin, radius, border width, bevel, spacing, font roles) live in `photon-web/design/tokens.json`. A script in `photon-web` generates CSS custom properties for the site and a Rust module committed in `photon-gui`, so the client builds without Node. A token changes in one place.
- **Skins.** The client has `classic` (default), `dark` and `light`. The website has `light` and `dark`. Skin and token names are the same on both sides.
- **Native video.** Plain video files from Photon's own servers (loading screen trailers) are decoded natively into a texture, in a later slice with a measured frame cost.

## Consequences

- The first slices (tokens, UI kit, home screen, loading screen, in-game menu) need no browser engine.
- The web surface is a spike, then a feature. Its cost in memory and start time is measured before it ships.
- Game pages are authored once, on the website.
