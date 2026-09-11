# P11 — Vanilla 26.2 kontrollü oyuncu hareketi doğrulaması

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

Sözleşme ayrıntıları [movement probe contract belgesindedir](movement-probe-contract.md). İlk teleport akışı [ayrı kayıttadır](vanilla-player-position-parity.md). Bu test resmi vanilla sunucusuyla yapıldı; Pumpkin koşusu bu baseline'a karıştırılmadı.
