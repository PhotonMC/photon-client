# P08/P11 — İlk oyuncu konumu düzeltmesi sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative ve peaceful ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example connection_probe -- 127.0.0.1:25566 RustPosition1
cargo run --offline --example connection_probe -- 127.0.0.1:25566 RustPosition2
```

İki bağımsız Rust koşusu aynı ilk konum düzeltmesi ve loaded bildirimini tamamladı:

| Gözlem | RustPosition1 | RustPosition2 |
| --- | ---: | ---: |
| login finished | `true` | `true` |
| configuration finished | `true` | `true` |
| play login | `true` | `true` |
| player position observations | `[{teleport_id:1, relative_flags:0}]` | `[{teleport_id:1, relative_flags:0}]` |
| player loaded sent | `true` | `true` |
| chunk packet | 22 | 21 |
| decoded chunk | 22 | 21 |
| chunk batch finished | 5 | 5 |
| received packet | 418 | 367 |

Probe çıktısında iki oturumda da packet ID `72` ile gelen teleport ID `1` ve boş relative flag görüldü. Rust, Java kaynak akışındaki sırayla teleport kabulü ve `move_player_pos_rot` cevabını yazdı; ardından `player_loaded` packet'ini gönderdi. Vanilla server logu iki kullanıcı için `logged in`, `joined the game` ve normal probe kapanışında `left the game` kayıtlarını verdi; protocol error veya timeout oluşmadı.

## Açık kapsam

Bu sonuç ilk server correction packet'inin decode edilip cevaplanmasını ve gerçek vanilla sunucusunun bağlantıyı kabul ettiğini doğrular. İlk konumun Java client entity state'ine uygulanması, input/tick hareketi, movement packet eşikleri, collision, teleport prediction, block action, inventory, kamera ve renderer henüz açık kalır. `player_loaded` bildiriminin probe zamanlaması level renderer hazır olma davranışının yerine geçmez.

Sözleşme ayrıntıları [player position contract belgesindedir](#sözleşme). Chunk/heightmap/light wire kanıtı [ayrı P08 kaydındadır](chunk-packet-conformance.md). Bu test resmi vanilla sunucusuyla yapıldı; Pumpkin koşusu bu baseline'a karıştırılmadı.
