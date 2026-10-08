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
Yapılan: `ServerStatus` JSON çözümü ve `handleStatusResponse` karşılığı (`voxel-game/src/
server_status.rs`: açıklama, oyuncular ve örnek, sürüm, favicon baytları; `Pinged`), base64 çözücü
(`voxel-core`), durum sorgusunun ping süresi (`round_trip_ms`); liste açılınca her sunucu kendi
iş parçacığında sorgulanır, satır MOTD (2 satır), oyuncu sayısı ya da kırmızı sürüm, ping çubukları,
ulaşılamaz/çözülemez metni ve çubuk/oyuncu ipuçlarını çizer. GUI atlasına `server_list` sprite'ları
eklendi (T86'da çubuklar ve oklar görünmüyordu). Oracle: `server-list-pings` 14 satır (Java yanıtı
`ServerStatus.CODEC` ile çözer). `VOXEL_MENU_SCREEN=servers` menüyü listeyle açar (yakalama için).
Sapma: el sıkışma ana bilgisayar adı yerine IP gönderir; açıklama `ComponentUtils.resolve` ile
temizlenmiyor; eski (1.6 öncesi) ping yok; Java'nın 5 iş parçacıklı havuzu yerine sunucu başına bir.
Açık: favicon çizimi ve kaydı → T86d.
