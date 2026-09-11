# P11 — Kontrollü oyuncu hareketi probe sözleşmesi

Bu dilim, ilk dünya yükleme bildirimi sonrasında vanilla server'a üç küçük mutlak konum/rotasyon paketi gönderir. Amaç, oyun oturumu açıkken serverbound hareket packet'inin doğru protokol alanlarıyla kabul edilmesini ve beklenmeyen teleport düzeltmesinin trace'e yansımasını ölçmektir.

## Java referansı

`LocalPlayer.sendPosition` içindeki hareket/rotasyon ayrımı ve `ServerboundMovePlayerPacket.PosRot` alan sırası incelendi. İlk server teleport cevabında `ClientPacketListener.handleMovePlayer` tarafından gönderilen cevap `onGround=false` ve `horizontalCollision=false` bayraklarını kullanır. Kontrollü probe, sonrasında serverbound hareket adımını ayakta duran başlangıç konumu için `onGround=true` ile yollar; bu adım input veya fizik oracle'ı değildir.

## Probe akışı

1. İlk `ClientboundPlayerPositionPacket` okunur; teleport kabulü ve mutlak `move_player_pos_rot` cevabı gönderilir.
2. İlk chunk batch'i tamamlanınca boş `player_loaded` packet'i gönderilir.
3. Sunucunun verdiği mutlak x konumuna sırasıyla `+0.25`, `+0.50` ve `+0.75` eklenerek üç `PosRot` packet'i gönderilir. y/z ve rotasyonlar sabit, flags byte'ı `onGround` için `1` olur.
4. Kısa drain penceresinde yeni clientbound oyuncu konumu düzeltmesi gözlenirse `player_position_observations` içine eklenir; bu server correction sonucu test çıktısıdır.

Bu probe'un `movement_packets_sent=3` sonucu packet gönderimini, ek teleport gözlemi olmaması ise bu kontrollü kısa izde server'ın correction göndermediğini gösterir. Server'ın otoriter fizik sonucu veya Java client'ın packet seçme eşiği çıkarılmaz.

## Sınır

Input snapshot'ı, tick zamanlaması, `2.0E-4` hareket eşiği, 20 tick hatırlatması, yerçekimi, collision, step-up, sprint, kamera ve görsel hareket bu dilimin dışındadır. Bunlar için aynı sabit başlangıç dünyasında Java client trace'i ve ayrı davranış oracle'ı gerekir.
