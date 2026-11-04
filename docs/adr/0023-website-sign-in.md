# ADR 0023: Website sign-in

- Status: proposed (2026-11-04); the owner can overrule it

## Context

A Play button on the website needs the backend to know which profile the browser belongs to, so
that the launch code it hands the game is bound to that profile ([ADR 0020](0020-accounts-and-launch-links.md)).
The only proof of a profile the backend accepts is the client's identity proof
([ADR 0022](0022-backend-services.md)), which never leaves the client. The browser has no
Minecraft token and must not get one.

A first draft had the site create a request that the client approves. It does not hold: an attacker
can create a request on the real backend, show its pairing code on their own page, and have the
victim's client approve it, because the code only shows that client and page look at the same
request, not that the request is the victim's. The attacker then polls and gets the victim's
session. The design below replaces it.

## Decision

The client opens the browser, and the browser proves it is the one that started the flow.

- The site sets a short-lived `HttpOnly` cookie holding a random `state` (32 hex digits) and opens
  `photon://signin?state=<same>`. The state grants nothing by itself.
- The client asks "Sign in to the Photon website as <name>?". On a yes it signs in to the backend
  with its identity proof, asks for a **web code** bound to the session's profile and that state
  (valid 60 seconds, single use), and opens the website address from its own configuration with
  `?code=<code>` in the default browser. The address is never taken from the link.
- The site's server exchanges the code together with the cookie's state for a website session, sets
  its own cookie, removes the code from the URL and answers with `Referrer-Policy: no-referrer`.
  The client's own backend session is never given to the browser.
- The backend consumes a code with one atomic delete and compares the state and the expiry after
  it, so a code burns on any attempt, including one with a wrong state.
- The link carries no secret, no token and no session. The parser for `photon://signin` is as strict
  as the launch one: one parameter, 32 hex digits, nothing else.

## Consequences

- An attacker who supplies a state of their own gets a code that lands in the victim's browser,
  where the cookie's state does not match, and never sees it. The attacker's own code fails the
  same way in the victim's browser. So neither session theft nor login CSRF works.
- If the person started the flow in a browser that is not the default, the cookie is missing and
  the exchange fails; the site says so.
- No Azure redirect is needed. Server-side Microsoft OAuth on the site stays a later option.
- Needs: a `web_codes` table with a create and an exchange call in the backend, the link parser and
  the confirmation screen in the client, and the site's Play page.
