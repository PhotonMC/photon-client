# P12 — Vanilla 26.2 boş elle use item doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Probe, ilk chunk ve `player_loaded` sonrasında boş ana elle açık alana `use_item` gönderdi.

```text
cargo run --offline --release --example use_item_probe -- 127.0.0.1:25566 RustUse5
cargo run --offline --release --example use_item_probe -- 127.0.0.1:25566 RustUse6
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustUse5 | RustUse6 |
| --- | ---: | ---: |
| use item packets sent | `1` | `1` |
| block changed ACK sequences | `[10]` | `[10]` |
| block update observations | `[]` | `[]` |
| chunk packet | `24` | `24` |
| decoded chunk | `24` | `24` |
| chunk batch finished | `6` | `6` |
| received packet | `101` | `102` |

Vanilla server, boş stack nedeniyle item action veya rotation snap sonucu üretmedi; block update alınmadı. Server logunda protocol error, timeout veya `UseItemPacket` rejection warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç `use_item` packet ID `67`, Java alan sırası, loaded oturum ACK'i ve boş ana elin dünya üzerinde değişiklik yapmamasını iki vanilla oturumunda doğrular. Dolu stack item kullanımı, tüketim/component güncellemeleri, iki el önceliği, swing ve client prediction açık kalır.

Sözleşme ayrıntıları [use item contract belgesindedir](use-item-contract.md). Boş elle blok yüzüne kullanım [ayrı kayıttadır](vanilla-use-item-on-parity.md); creative placement [ayrı kayıttadır](vanilla-creative-placement-parity.md).
