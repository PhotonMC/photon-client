# P08/P11 — İlk oyuncu konumu düzeltmesi sözleşmesi

Bu dilim, vanilla 26.2 sunucusunun ilk dünya girişinde gönderdiği oyuncu konumu paketini okuyup Java client'ın zorunlu teleport kabulü ve konum/rotasyon cevabını üretir. Amaç, oyuncu entity'si için gerçek hareket veya çarpışma simülasyonu kurmadan ilk oyun protokolü el sıkışmasını sınamaktır.

## Java referansı

Kaynak incelemesinde `ClientboundPlayerPositionPacket`, `PositionMoveRotation` ve `Relative` alan sırası; `ClientPacketListener.handleMovePlayer` içindeki düzeltme, teleport kabulü ve takip eden `PosRot` cevabı; `ServerboundPlayerLoadedPacket` ile `notifyPlayerLoaded` çağrısı incelendi. `GameProtocols` kayıt sırası protokol 776 packet ID'leriyle birlikte kontrol edildi. Orijinal kaynak depoya kopyalanmadı.

## Wire sırası

- Clientbound player position packet'i teleport ID VarInt'i, üç konum f64'ü, üç delta-movement f64'ü, yaw/pitch f32'leri ve relative flag'lerini taşıyan i32'den oluşur.
- Relative konum ve rotasyon flag'leri mevcut oyuncu değerlerine eklenir; pitch sonucu Java'nın `-90..90` clamp sınırına alınır.
- Java client önce `accept_teleportation` packet'ini teleport ID ile, sonra mutlak konum ve rotasyonu taşıyan `move_player_pos_rot` packet'ini gönderir.
- İlk dünya yükleme bildirimi `player_loaded` boş payload'lı packet'tir. Gerçek Java client bunu level yükleme/derleme hazır olduğunda gönderir.

Rust parser packet'i tam tüketir, teleport ID ve relative flag'lerini trace'e kaydeder, konum/rotasyonu relative kuralla çözüp iki serverbound cevabı gönderir. Headless probe'da renderer olmadığı için `player_loaded`, ilk chunk ve chunk batch tesliminden sonra gönderilen protokol işaretidir; bu zamanlama Java renderer hazır olma eşitliği değildir.

## Sınır

Bu dilim input snapshot'ı, tick başına hareket paket seçimi, yerçekimi, collision, prediction, kamera veya görsel oyuncu state'i uygulamaz. Bunlar P09–P16 ve ilgili hareket senaryolarının kapsamıdır. Canlı vanilla kabulü packet akışının çalıştığını gösterir; tek başına oynanabilir istemci veya tam Java davranış eşitliği değildir.
