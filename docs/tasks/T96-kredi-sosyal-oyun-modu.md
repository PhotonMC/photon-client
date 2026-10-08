# T96 — Kredi ekranı, sosyal etkileşimler ve oyun modu seçici

**Amaç:** `WinScreen` (end şiiri ve kredi kaydırması; `GameEvent` WIN_GAME → `ClientCommand`
PERFORM_RESPAWN), `SocialInteractionsScreen` (oyuncu listesi, gizleme; P key), `GameModeSwitcherScreen`
(F3+F4, `ChangeGameMode` komutu).
**Bağımlılık:** — . **Boy:** M.

## Oku
- Java: `client/gui/screens/WinScreen.java`, `client/gui/screens/social/*.java`,
  `client/gui/screens/debug/GameModeSwitcherScreen.java`, `ClientPacketListener.handleGameEvent`.
- Rust: `crates/photon-net/src/connection/play/incoming.rs` (oyun olayları), tablist oyuncu bilgisi.

## Yapılacaklar
1. Kredi: `texts/end.txt`, `texts/credits.json` biçimi, kaydırma hızı, bitince paket.
2. Sosyal ekran: sekmeler, oyuncu satırları, gizleme (sohbet filtresine bağlanır).
3. Oyun modu seçici.
4. G tarifi: üç operation.

## Oracle
G.

## Kabul
`CHECK OK`.

## Kapsam dışı
Raporlama ekranları (çevrimiçi servis).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
