# P12 — Survival block-break başlatma ve iptal sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun aynı hedef için `START_DESTROY_BLOCK` gönderip hemen ardından `ABORT_DESTROY_BLOCK` ile kazmayı iptal etmesini gerçek vanilla server üzerinde sınar. Amaç hedefin kırılmadığını, sequence ACK sırasını ve server'ın actor dışına gönderdiği break-progress davranışını ayırmaktır.

## Java referansı

`ServerboundPlayerActionPacket.Action` sırasına göre `START_DESTROY_BLOCK` ordinal `0`, `ABORT_DESTROY_BLOCK` ordinal `1` ve `STOP_DESTROY_BLOCK` ordinal `2`'dir. Serverbound packet ID'si `41`'dir. `ServerGamePacketListenerImpl.handlePlayerAction` start/abort/stop action'larını `ServerPlayerGameMode.handleBlockBreakAction`'a aktarır ve sequence'i ACK kuyruğuna alır.

Survival start, hedefin destroy progress değerini hesaplayıp `ServerLevel.destroyBlockProgress` ile progress başlangıcını yayınlar. Abort aynı hedefte progress'i `-1` ile temizler ve dünya block state'ini değiştirmez. `ServerLevel.destroyBlockProgress` kendi action'ını yapan oyuncuyu hedef listesinden çıkardığı için actor bağlantısı `ClientboundBlockDestructionPacket` almaz; yerel client'ın kendi break animasyonu ayrı prediction kapsamıdır.

## Probe girdisi ve akışı

- Game mode: `SURVIVAL`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Direction: `UP`, ID `1`
- Start sequence: `6`
- Abort sequence: `7`
- Ana el: boş

İlk `player_loaded` sonrasında Rust start ve abort packet'lerini aynı oturumda yollar. Vanilla server ACK kuyruğunda en yüksek sequence'i `[7]` olarak döndürür; hedef için block update üretmez ve hedef taş olarak kalır.

## Sınır

Bu dilim survival kazma süresini, tick aralıklarını, `STOP_DESTROY_BLOCK` ile tamamlamayı, delayed destroy yolunu, ray hit üretimini, client-side break prediction'ı ve başka oyuncuya gönderilen destruction progress akışını kapsamaz.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Spawn aramasının sabit bir blok üzerinde kalması için geçici dünyada y=`100` seviyesinde geniş taş platformu ve `(100,101,100)` world spawn'ı kullanıldı. JAR, dünya ve server çıktıları depoya alınmadı.

```text
cargo run --offline --release --example survival_break_abort_probe -- 127.0.0.1:25566 RustSurv1
cargo run --offline --release --example survival_break_abort_probe -- 127.0.0.1:25566 RustSurv2
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustSurv1 | RustSurv2 |
| --- | ---: | ---: |
| player action packets sent | `2` | `2` |
| block changed ACK sequences | `[7]` | `[7]` |
| block destruction observations | `[]` | `[]` |
| block update observations | `[]` | `[]` |
| chunk packet | 27 | 24 |
| decoded chunk | 27 | 24 |
| chunk batch finished | 7 | 6 |
| received packet | 118 | 112 |

Server logundaki player konumları RustSurv1 için `(96.5,101.0,91.5)`, RustSurv2 için `(98.5,101.0,110.5)` idi; hedefler sırasıyla `(96,100,91)` ve `(98,100,110)` oldu. Server console `execute if block ... minecraft:stone` kontrolleri iki hedefin de abort sonrasında sağlam kaldığını gösterdi. Break-progress listelerinin boş olması Java `ServerLevel.destroyBlockProgress` actor oyuncusunu yayın alıcılarından çıkardığı için beklenen sonuçtur. Vanilla logunda protocol error, timeout veya action warning oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server üzerinde survival start/abort action ordinal'larını, aynı hedefte iki packet akışını, sequence ACK birleştirmesini ve dünya değişikliği olmamasını doğrular. Survival progress tick'leri, stop/full break, delayed destroy, ikinci oyuncuya progress yayını, client prediction/rollback, ray hit ve item/tool etkileri açık kalır.

Sözleşme ayrıntıları [survival break abort contract belgesindedir](#sözleşme). Creative break [ayrı kayıttadır](block-break-conformance.md); creative placement [ayrı kayıttadır](creative-placement-conformance.md).
