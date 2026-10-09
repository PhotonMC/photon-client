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
Yapıldı: `WinScreen` (şiir + kredi + sonkredi) `screen/credits.rs`; oyun içi bağlama (`GameEvent` 4 → `Controls::credits`, Esc/bitiş → `PerformRespawn`); vinyet için yeni `GuiBlend::Vignette` boru hattı; kesirli kayan menü arka planı; oracle `credits_screen` (parity yeşil).
Java'dan sapma: şiirin arka planı uç portalı shader'ıdır, çizilmez (GPU siyah, günlük Java'nın beyaz dolgusunu korur); `§k` karıştırılmış harfler çizilmez; müzik (`Musics.CREDITS`) ve `isPauseScreen` (kredi tek oyunculuda duraklatır) yok.
Açık: uç portalı shader'ı (iki katman), kredi müziği, sonkredi anlatıcı metni, poem=false yolunun oyun içi tetiği (ana menüden "Credits" T85'te).
