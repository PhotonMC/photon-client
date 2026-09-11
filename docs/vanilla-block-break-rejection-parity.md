# P12 — Vanilla 26.2 menzil dışı blok kırma doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example block_break_rejection_probe -- 127.0.0.1:25566 RustReject1
cargo run --offline --example block_break_rejection_probe -- 127.0.0.1:25566 RustReject2
```

İki bağımsız koşunun ortak sonucu:

| Gözlem | RustReject1 | RustReject2 |
| --- | ---: | ---: |
| play login | `true` | `true` |
| player loaded sent | `true` | `true` |
| player action packets sent | 1 | 1 |
| block changed ACK sequences | `[3]` | `[3]` |
| block update observations | `[]` | `[]` |

Vanilla server menzil dışı hedef için block-break state'i veya block update üretmedi; sequence ACK'i yine `[3]` olarak döndürdü. Server logunda protocol error, timeout, mismatch veya action warning oluşmadı.

## Açık kapsam

Bu sonuç yalnızca server menzil reddi ile sequence ACK davranışını doğrular. Client ray/hit sonucu, survival süreleri, spawn protection, placement, item use ve prediction/rollback açık kalır.

Sözleşme ayrıntıları [block-break rejection contract belgesindedir](block-break-rejection-contract.md). Pozitif creative akış [ayrı kayıttadır](vanilla-block-break-parity.md).
