# T86c — Sunucu ekleme, düzenleme, silme ve doğrudan bağlanma

**Amaç:** Sunucu listesinin (T86) kalan düğmeleri: `ManageServerScreen` (Add/Edit: ad ve adres
kutuları, kaynak paketi döngü düğmesi, Done'ın adres geçerliyken uyanması), `DirectJoinServerScreen`
(adres kutusu, son adres `options.txt` `lastServer`), Delete için `ConfirmScreen` (T03), ve
`JoinMultiplayerScreen`'in geri çağrıları (`addServerCallback`: gizli sunucuyu açma,
`editServerCallback`, `directJoinCallback`: gizli olarak ekleme).
**Bağımlılık:** T86. **Boy:** M.

## Oku
- Java: `client/gui/screens/ManageServerScreen.java`, `client/gui/screens/DirectJoinServerScreen.java`,
  `JoinMultiplayerScreen` (geri çağrılar), `client/multiplayer/resolver/ServerAddress.java` (`isValidAddress`).
- Rust: `crates/voxel-render/src/gui/screen/servers.rs`, `crates/voxel-game/src/servers.rs`,
  `crates/voxel-render/src/gui/widgets/edit_box.rs`, `crates/voxel/src/menu.rs` (`servers_action`).

## Yapılacaklar
1. İki ekran (G tarifi) ve menüye bağlanmaları; silme onayı.
2. `ServerList` geri çağrıları ve kaydetme.

## Oracle
G: `manage_server_screen`, `direct_join_screen` (yazılan metinler, Done'ın durumu).

## Kabul
`CHECK OK`; menüden eklenen sunucuya bağlanılır.

## Kapsam dışı
Durum sorgusu (T86b), LAN.

## Belgeler
`docs/gui-oracle.md` satırları.

## Devir notu
