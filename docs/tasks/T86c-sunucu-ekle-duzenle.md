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
- Rust: `crates/photon-render/src/gui/screen/servers.rs`, `crates/photon-game/src/servers.rs`,
  `crates/photon-render/src/gui/widgets/edit_box.rs`, `crates/photon/src/menu.rs` (`servers_action`).

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
Yapılan: `ServerFormScreen` (`photon-render/src/gui/screen/server_form.rs`: Add/Edit/Direct;
kutular, ipucu, kaynak paketi döngüsü, Done'ın `isValidAddress` ile uyanması, Tab, tıklama),
`is_valid_address` (Guava `HostAndPort` kuralları), `options.txt` `lastServer`; menüde ekle/düzenle/
sil (onay) ve doğrudan bağlan (gizli sunucu olarak kaydedilir). Oracle `server_form` 21 satır.
Sapma: IDN'nin ASCII dışı ad denetimi yok (yalnız etiket uzunluğu); form ve onay ekranı pencere
boyutu değişince yeniden kurulmuyor; onay ekranının arkasında panorama örtüsü yok.
