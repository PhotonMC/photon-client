# P11 — Kontrollü oyuncu hareketi probe sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative ve peaceful ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example movement_probe -- 127.0.0.1:25566 RustMove3
cargo run --offline --example movement_probe -- 127.0.0.1:25566 RustMove4
```

İki bağımsız koşunun çıktısı:

| Gözlem | RustMove3 | RustMove4 |
| --- | ---: | ---: |
| play login | `true` | `true` |
| ilk teleport gözlemi | `[{teleport_id:1, relative_flags:0}]` | `[{teleport_id:1, relative_flags:0}]` |
| player loaded sent | `true` | `true` |
| movement packets sent | 3 | 3 |
| chunk packet | 21 | 24 |
| decoded chunk | 21 | 24 |
| chunk batch finished | 5 | 6 |
| received packet | 394 | 428 |

Her iki koşuda ilk teleport düzeltmesinden sonra üç `move_player_pos_rot` paketi gönderildi ve drain penceresinde ikinci bir teleport düzeltmesi görülmedi. Vanilla server logunda iki kullanıcı için normal `logged in`, `joined the game` ve probe kapanışında `left the game` kayıtları vardı; protocol error veya timeout oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server'a kontrollü movement packet gönderiminin ve kısa izde correction alınmamasının kanıtıdır. Java `LocalPlayer` input/tick paket seçimi, hareket eşiği, 20 tick hatırlatması, yerçekimi/collision sonucu, server state snapshot'ı, hareket senkronizasyonu ve oynanabilir client henüz doğrulanmış değildir.

Sözleşme ayrıntıları [movement probe contract belgesindedir](#sözleşme). İlk teleport akışı [ayrı kayıttadır](player-position-conformance.md). Bu test resmi vanilla sunucusuyla yapıldı; Pumpkin koşusu bu baseline'a karıştırılmadı.
