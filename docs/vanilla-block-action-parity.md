# P12 — Vanilla 26.2 blok eylemi sequence ACK doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative ve peaceful ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example action_probe -- 127.0.0.1:25566 RustAction3
cargo run --offline --example action_probe -- 127.0.0.1:25566 RustAction4
```

İki bağımsız koşunun çıktısı:

| Gözlem | RustAction3 | RustAction4 |
| --- | ---: | ---: |
| play login | `true` | `true` |
| ilk teleport gözlemi | `[{teleport_id:1, relative_flags:0}]` | `[{teleport_id:1, relative_flags:0}]` |
| player loaded sent | `true` | `true` |
| player action packets sent | 1 | 1 |
| block changed ACK sequences | `[1]` | `[1]` |
| chunk packet | 24 | 24 |
| decoded chunk | 24 | 24 |
| chunk batch finished | 6 | 6 |
| received packet | 378 | 402 |

Her iki koşuda Rust, `ABORT_DESTROY_BLOCK` için action `1`, packed `BlockPos.ZERO`, `DOWN` ve sequence `1` alanlarını gönderdi. Vanilla server aynı sequence için ACK döndürdü; server logunda protocol error, timeout veya action mismatch warning oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server'ın sequence ACK akışını ve block action packet wire sırasını doğrular. Reach/hit sonucu, gerçek block break veya placement, item use, prediction/rollback, client world state, envanter ve görsel/ses feedback'i henüz açık kalır.

Sözleşme ayrıntıları [block action contract belgesindedir](block-action-contract.md). İlk oyuncu konumu [ayrı kayıtta](vanilla-player-position-parity.md), kontrollü hareket [ayrı kayıtta](vanilla-movement-probe-parity.md) tutulur. Bu test resmi vanilla sunucusuyla yapıldı; Pumpkin koşusu bu baseline'a karıştırılmadı.
