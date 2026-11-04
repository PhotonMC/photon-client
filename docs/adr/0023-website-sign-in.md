# ADR 0023: Website sign-in

- Status: proposed (2026-11-04); the owner can overrule it

## Context

A Play button on the website needs the backend to know which profile the browser belongs to, so
that the launch code it hands the game is bound to that profile ([ADR 0020](0020-accounts-and-launch-links.md)).
The only proof of a profile the backend accepts is the client's identity proof
([ADR 0022](0022-backend-services.md)), which never leaves the client. The browser has no
Minecraft token and must not get one.

## Decision

The browser asks the backend for a **sign-in request** and the client approves it.

- The site creates a request: a random id (at least 128 bits) and a short pairing code of six
  characters, both held by the backend for at most 60 seconds. The site shows the pairing code and
  opens `photon://signin?request=<id>`; the id is the only thing in the link.
- The client reads the request, shows a screen that says what is being approved ("Sign in to the
  Photon website as <name>?") with the pairing code the site shows, and signs in to the backend with
  its own identity proof only when the player confirms. It then approves the request with that
  session. Nothing is approved without the player pressing the button, and the code on the screen
  must be compared with the one on the site.
- The browser polls its request with a secret only it holds (held in the page, never in a URL) and
  receives a website session when the request is approved. A request is approved at most once and
  consumed by one atomic delete.
- The link carries no secret, no token and no session. The parser for `photon://signin` is as strict
  as the launch one: one parameter, a 32-hex id, nothing else.

## Consequences

- An attacker who sends a victim a `photon://signin` link for the attacker's own request gets a
  session for the victim only if the victim compares nothing and confirms. The pairing code on the
  confirmation screen and the 60 second life are the mitigation; the screen says plainly that the
  code must match the website.
- No Azure redirect is needed. Server-side Microsoft OAuth on the site stays a later option.
- Needs: a request table and three calls in the backend (create, approve, poll), the link parser
  and the confirmation screen in the client, and the site's Play page. None is built yet.
