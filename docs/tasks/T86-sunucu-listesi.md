# T86 — Sunucu listesi ve `servers.dat`

**Amaç:** `JoinMultiplayerScreen` + `ServerSelectionList`: `servers.dat` (NBT) okuma/yazma, liste
düzeni, seçim, çift tık/Join, Add/Edit/Delete/Direct Connect/Refresh düğmeleri. Durum sorgusu
(MOTD, ping, oyuncu sayısı, favicon) T86b'de; bu görevde satırlar "pinging" hâlinde çizilir.
**Bağımlılık:** T85. **Boy:** L.

## Oku
- Java: `client/gui/screens/multiplayer/JoinMultiplayerScreen.java`,
  `client/gui/screens/multiplayer/ServerSelectionList.java`, `client/multiplayer/ServerList.java`,
  `client/multiplayer/ServerData.java`, `client/gui/screens/ManageServerScreen.java`,
  `client/gui/screens/DirectJoinServerScreen.java`, `client/multiplayer/resolver/ServerAddress.java`.
- Rust: `crates/voxel-core/src/nbt*` (okuyucu/yazıcı), `crates/voxel-render/src/gui/options/list*`
  (liste widget'ı), T85 durum makinesi.

## Yapılacaklar
1. `servers.dat` NBT modeli (sıra, `hidden`, `acceptTextures`, ikon base64).
2. Liste ekranı + Add/Edit (`ManageServerScreen`) + Direct Connect; seçilen sunucuya bağlan (T87).
3. G tarifi: `join_multiplayer_screen`, `manage_server_screen`, `direct_join_screen`.

## Oracle
G: üç ekran; `ServerList` NBT gidiş-dönüşü `nbt` ailesinde.

## Kabul
`CHECK OK`; listeye eklenen Pumpkin sunucusuna menüden bağlanılır.

## Kapsam dışı
Durum sorgusu (T86b), LAN keşfi.

## Belgeler
`docs/gui-oracle.md` satırları.

## Devir notu
