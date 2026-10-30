# ADR 0012: Bidirectional text and Arabic shaping

- Status: accepted (2026-10-30)

## Context

The Java client reorders right-to-left text before it draws it. Each string goes through an Arabic shaping pass (letters in joined forms, lam-alef ligatures, no digit or tashkeel changes beyond that pass) and the Unicode bidirectional algorithm at the level the font code asks for; formatted text is reordered per styled run, and the language decides the base direction. Strings in Arabic or Hebrew servers, signs, books and chat reach this path whenever the language is right-to-left or the text itself contains right-to-left characters.

Photon draws such text in logical order today. The result differs from the Java client for every string with right-to-left characters, in the glyph forms and in the order.

## Decision

- The behaviour is implemented, not skipped: it is observable draw output.
- The Unicode algorithms come from a maintained crate, not from a port of Java's ICU code. Candidates: `unicode-bidi` (UAX #9) for reordering, and a shaping implementation limited to the Arabic joining forms the Java call uses (a table-driven pass of our own if no crate matches the Java option set exactly). The choice between them is made by the oracle: the crate that agrees with the Java client on the case set wins.
- The reordering sits in the text layout layer, below the draw code and above the glyph lookup, so every consumer (labels, edit boxes, signs, chat) gets it from one place.
- Edit boxes keep their cursor and selection in logical order; only the draw order changes.

## Checking

- Original strings (not taken from game files) with Arabic, Hebrew, mixed-direction, digit-run and formatting-code content go through a parity case that asks the Java client for the shaped and reordered code point sequence per styled run.
- Mutation check: a case row with a joined form or a mixed run must fail when the pass is disabled.

## Consequences

- One new dependency, with its licence checked against the project's policy before it is added.
- The glyphs for Arabic and Hebrew come from the existing unifont provider; this ADR only changes the order and forms of the code points handed to it.
- Input method composition stays a separate slice.
