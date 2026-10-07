# T86b — Sunucu durum sorgusu (MOTD, ping, favicon)

**Amaç:** `ServerStatusPinger`: durum isteği ve ping (`crates/voxel-net/src/status.rs` kodekleri
var), yanıt JSON'u (`ServerStatus`: açıklama bileşeni, oyuncular, sürüm, favicon), ping çubukları,
uyumsuz sürüm işareti, favicon dokusu; satırın Java'daki çizimi.
**Bağımlılık:** T86. **Boy:** M.

## Oku
- Java: `client/multiplayer/ServerStatusPinger.java`, `network/protocol/status/ServerStatus.java`,
  `client/gui/screens/FaviconTexture.java`, `ServerSelectionList.OnlineServerEntry`
  (`extractContent`, ping simgesi seçimi, ipuçları).
- Rust: `crates/voxel-net/src/status.rs`, T86 liste satırı.

## Yapılacaklar
1. Arka plan iş parçacığında sorgu (zaman aşımı Java'nınki), sonuç satıra.
2. Ping çubuğu eşikleri, oyuncu listesi ipucu, sürüm uyumsuzluğu.
3. G tarifi: durum dolu satırlarla `join_multiplayer_screen` satırları.

## Oracle
G (satır çizimi); `ServerStatus` JSON çözümü `json` ailesinde; canlı Pumpkin'e sorgu
`server-conformance`.

## Kabul
`CHECK OK`; listede Pumpkin'in MOTD'si ve pingi.

## Kapsam dışı
LAN, eski (1.6 öncesi) ping.

## Belgeler
`docs/gui-oracle.md`; yeni `server-conformance` kaydı.

## Devir notu
