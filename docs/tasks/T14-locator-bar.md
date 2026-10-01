# T14 — `TrackedWaypoint` + `LocatorBar`

**Amaç:** `ClientboundTrackedWaypointPacket` (waypoint izleme) ve HUD'daki konum çubuğu (`LocatorBar`, deneyim çubuğunun yerine geçer).
**Bağımlılık:** yok (yön hesabı entity gerektirir; yalnız `Vec3i`/`Chunk`/`Azimuth` waypoint'leri). **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/network/protocol/game/ClientboundTrackedWaypointPacket.java`, `.data/minecraft/source/net/minecraft/world/waypoints/{TrackedWaypoint,TrackedWaypointManager,Waypoint,WaypointStyleAssets}.java`, `.data/minecraft/source/net/minecraft/client/gui/contextualbar/{LocatorBar,ContextualBar}.java`, `.data/minecraft/source/net/minecraft/client/waypoints/ClientWaypointManager.java`, `.data/minecraft/source/net/minecraft/client/gui/Hud.java` (`extractContextualBarAndLevel`/`nextContextualInfoState`: `grep -n "LocatorBar\|contextual"`), waypoint sprite'ları `assets/minecraft/textures/gui/sprites/hud/locator_bar_*` ve `waypoint_styles/*` (`atlas.rs` klasör listesi).
- Rust: `crates/voxel-render/src/gui/hud.rs` (deneyim çubuğu çizimi), `crates/voxel-net/src/connection/play.rs` (paket dağıtımı) + `protocol.rs` ayrıştırıcı kalıbı (`parse_player_info_update`), paket kimliği = `network/protocol/game/GameProtocols.java` clientbound play kaydındaki 0 tabanlı sıra (sayarken mevcut sabitle doğrula: `ADD_ENTITY` = 1, `BOSS_EVENT` = 9); kimliği bir bayt testiyle sabitle.

## Yapılacaklar
1. Paket ayrıştırma (`Operation`: TRACK/UNTRACK/UPDATE; waypoint türleri: `Vec3i`, `Chunk`, `Azimuth`, `Empty`; ikon/stil, renk) + `ClientWaypointManager` durumu (`PlaySession` içinde).
2. `LocatorBar` yerleşimi: hangi koşulda deneyim çubuğunun yerine geçer (Java `Hud` kuralı: oyuncunun `locator_bar` oyun kuralı/`WAYPOINT_TRANSMIT_RANGE`; sunucudan gelen veriyle), açı → x konumu formülü (`TrackedWaypoint.yawAngleToCamera`), ok simgeleri (yukarı/aşağı), mesafeye göre sprite ölçeği.
3. Çizim yalnız mantıksal yerleşim; harita ve entity waypoint'leri (`Entity` kaynaklı) entity yokken `Empty` gibi davranır (belgele).

## Oracle (G)
`locator_bar`: gerçek `LocatorBar.extractBackground/extractRenderState` (sahte `TrackedWaypoint`'ler; kamera yaw/pozisyonu satırda). Satırlar: önde/arkada/yanda, yukarı/aşağı ok, mesafe sprite'ı, 0/çok waypoint. Paket için `wire` ailesine bayt satırı (Java `ClientboundTrackedWaypointPacket.STREAM_CODEC` ile).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Harita waypoint'leri, entity kaynaklı waypoint'ler (T20+ sonrası).

## Belgeler
`docs/gui-oracle.md`; `docs/remaining-work.md` "TrackedWaypoint" satırı.

## Devir notu
(boş)
