# T45 — Saldırı gücü göstergesi ve `PlayerCombat*` paketleri

**Amaç:** `AttackIndicator` HUD'ı (nişangâh/hotbar), `Player.getAttackStrengthScale` istemci kolu ve `ClientboundPlayerCombat{Kill,Enter,End}` paketleri; ölüm mesajı `CombatTracker` yalnız sunucuda.
**Bağımlılık:** T26. **Boy:** S. **Türler:** oyuncu.

## Oku
- Java: `client/gui/Hud.java`.
- Rust: `crates/voxel-game/src/attack.rs` (saldırı gücü/bekleme), `crates/voxel-render/src/gui/hud.rs` (HUD), `crates/voxel-net/src/connection/play.rs` (paket işleme).

## Yapılacaklar
1. Saldırı göstergesi oracle'ı (`gui` ailesi: `AttackIndicator` çizimi, gerçek `Hud`).
2. `PlayerCombatKill` ile ölüm ekranı mesajı bağı (ölüm ekranı var; mesaj kaynağı).
3. Tepme istemci tarafında hesaplanmaz (sunucu hareket paketi gönderir): doğrula, görev kapsamında değil.

## Oracle
`GuiOracle.java` (`gui` ailesi, `docs/tasks/README.md` "GUI oracle tarifi (G)"): gerçek `Hud`/`Gui` ile `AttackIndicator` çizimi; paketler için gerçek `ClientPacketListener` işleyicileri.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Hasar yönü göstergesi, ses.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Küçük; bağımsız. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
