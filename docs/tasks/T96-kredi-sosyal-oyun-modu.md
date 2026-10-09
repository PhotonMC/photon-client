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
- Bölündü: oyun modu seçici bitti (aşağıda); kredi ekranı T96b, sosyal etkileşimler T96c.
- Oyun modu seçici (F3+F4, `client-parity`): `photon-render/gui/screen/game_mode.rs`, `controls/game_mode_switcher.rs`; Java'nın gözlenen kuralları korundu (seçim sprite'ı bir geçiş sonra, fare yalnız kımıldadıktan sonra seçer, F4 fareyi sıfırlar). Oturum: `previous_game_mode` (giriş/yeniden doğma/oyun olayı), `may_change_game_mode` (izin düzeyi ≥ 2), `ChangeGameMode` paketi (sunucuya, kimlik 5).
- Açık: F3/F4 tuşları sabit (hata ayıklama tuşları yeniden atanamıyor, T93); F3'ü bırakmanın geçişi ve izinsiz iletinin sohbet satırı yalnız elle doğrulanır (oracle ekranı çizer; tuş akışı ve paket değil).
