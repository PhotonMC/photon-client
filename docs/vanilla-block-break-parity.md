# P12 — Vanilla 26.2 yaratıcı blok kırma doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example block_break_probe -- 127.0.0.1:25566 RustBreak1
cargo run --offline --example block_break_probe -- 127.0.0.1:25566 RustBreak2
cargo run --offline --example block_break_probe -- 127.0.0.1:25566 RustBreak3
```

Üç bağımsız koşunun sonuçları:

| Gözlem | RustBreak1 | RustBreak2 | RustBreak3 |
| --- | ---: | ---: |
| play login | `true` | `true` | `true` |
| player loaded sent | `true` | `true` | `true` |
| player action packets sent | 1 | 1 | 1 |
| block changed ACK sequences | `[2]` | `[2]` | `[2]` |
| block update observations | `[{x:24,y:76,z:-30,state_id:0}]` | `[{x:25,y:66,z:-39,state_id:0}]` | `[{x:27,y:71,z:-36,state_id:0}]` |
| chunk packet | 21 | 21 | 22 |
| decoded chunk | 21 | 21 | 22 |
| chunk batch finished | 5 | 5 | 5 |
| received packet | 406 | 393 | 322 |

Her üç koşuda `block_update` konumu, server logundaki spawn player konumunun bir blok altındaki hedefle aynıydı; server creative destroy sonrası state ID `0` döndürdü. Vanilla server logunda offline-mode uyarıları dışında protocol error, timeout veya block action warning oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server üzerinde hedef menzil kontrolünden geçen creative instant-break, sequence ACK ve block update dönüşünü doğrular. Survival destroy progress, abort/stop sırası, ray hit sonucu, placement, item use, client world state, prediction/rollback, inventory ve görsel/ses feedback'i açık kalır.

Sözleşme ayrıntıları [block break contract belgesindedir](block-break-contract.md). Önceki harmless ACK probe'u [ayrı kayıttadır](vanilla-block-action-parity.md). Bu test resmi vanilla sunucusuyla yapıldı; Pumpkin koşusu bu baseline'a karıştırılmadı.
