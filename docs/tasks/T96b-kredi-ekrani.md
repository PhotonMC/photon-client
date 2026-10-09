# T96b — Kredi ekranı

**Amaç:** `WinScreen` (end şiiri ve kredi kaydırması; `GameEvent` WIN_GAME → `ClientCommand` PERFORM_RESPAWN).
**Bağımlılık:** T85. **Boy:** M. [T96](T96-kredi-sosyal-oyun-modu.md)dan ayrıldı.

## Oku
Java: `client/gui/screens/WinScreen.java`, `ClientPacketListener.handleGameEvent` (WIN_GAME). Rust: `crates/photon-net/src/connection/play/incoming/player.rs` (oyun olayları).

## Yapılacaklar
`texts/end.txt` ve `texts/credits.json` biçimi, kaydırma hızı, bitince paket (G tarifi: `credits_screen`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
(boş)
