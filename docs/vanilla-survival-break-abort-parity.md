# P12 — Vanilla 26.2 survival block-break abort doğrulaması

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

Sözleşme ayrıntıları [survival break abort contract belgesindedir](survival-break-abort-contract.md). Creative break [ayrı kayıttadır](vanilla-block-break-parity.md); creative placement [ayrı kayıttadır](vanilla-creative-placement-parity.md).
