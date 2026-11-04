# ADR 0021: Client UI technology and design tokens

- Status: proposed (2026-11-04); the web surface part waits on the project owner's confirmation

## Context

The new menu flow ([ADR 0018](0018-photon-platform.md)) needs cards, grids, sidebars, scrolling lists and an avatar preview. The website and the client must look like one design: the website uses light and dark skins, and the client uses the game-like "Classic" skin. Some panels will play videos: server trailers on the loading screen, and YouTube videos. YouTube allows playback only through its official embedded player, which needs a browser engine. Plain video files (WebM/MP4) from Photon's own servers can be decoded without a browser.

## Decision

- **Native UI on `photon-gui`.** Photon's screens are drawn by the client's own GUI code and renderer, as the game screens are. No browser engine is needed to start the client or to show any menu.
- **One token file.** Design tokens (colors per skin, radius, border width, bevel, spacing, font roles) live in one source file in the website repository. A generator writes CSS custom properties for the website and a Rust module for the client. A token changes in one place.
- **Skins.** The client has `classic` (default), `dark` and `light`. The website has `light` and `dark`. Skin names and token names are the same on both sides.
- **Native video.** Video files from Photon servers and the backend (loading screens, server pages) are decoded natively into a texture. The decoder is chosen in its own slice and measured for frame cost.
- **Proposed: web surface.** For YouTube and other web content, an optional off-screen browser surface (a CEF-style engine that renders into a texture) is downloaded only when a panel first needs it. The client works fully without it. This part waits on the owner's confirmation and on a measurement of memory and startup cost.

## Consequences

- The first milestone (launch link, loading screen, home screen, server list, menu) needs neither video nor the web surface.
- The token generator is a small script in `photon-web`. Its Rust output is committed in this repository, so the client builds without Node.
