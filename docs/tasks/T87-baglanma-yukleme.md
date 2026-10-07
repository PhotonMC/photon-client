# T87 — Bağlanma ve dünya yükleme ekranları

**Amaç:** `ConnectScreen` (çözümleme, bağlanma, iptal, hata → `DisconnectedScreen`) ve
`LevelLoadingScreen` (chunk yükleme ilerlemesi, "Loading terrain") ekranları; ağ işi pencere
döngüsünü bloklamadan arka planda.
**Bağımlılık:** T85. **Boy:** M.

## Oku
- Java: `client/gui/screens/ConnectScreen.java`, `client/gui/screens/LevelLoadingScreen.java`,
  `client/multiplayer/ClientHandshakePacketListenerImpl.java` (durum metinleri),
  `client/multiplayer/LevelLoadTracker.java`.
- Rust: `crates/voxel-net/src/connection/session.rs`, `crates/voxel/src/play.rs` (`run_play`),
  `crates/voxel-render/src/gui/screen/disconnected.rs`.

## Yapılacaklar
1. Bağlanmayı ayrı iş parçacığına al; ekran durum metnini (Connecting/Logging in/Joining world) gösterir.
2. Yükleme ekranının bitiş koşulu (Java'nın `LevelLoadTracker`'ı) ve ilerleme çizimi.
3. G tarifi: `connect_screen`, `level_loading_screen`.

## Oracle
G: iki ekran.

## Kabul
`CHECK OK`; menüden bağlanırken ekranlar görünür (`shot`).

## Kapsam dışı
Yeniden yapılandırma ekranı (M9).

## Belgeler
`docs/gui-oracle.md`, `docs/live-play-session.md`.

## Devir notu
