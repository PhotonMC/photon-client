# P12 — Vanilla 26.2 survival block-break stop doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Spawn araması için geçici dünyada y=`100` seviyesinde geniş dirt platformu ve `(100,101,100)` world spawn'ı kullanıldı. JAR, dünya ve server çıktıları depoya alınmadı.

```text
cargo run --offline --release --example survival_break_stop_probe -- 127.0.0.1:25566 RustStop1
cargo run --offline --release --example survival_break_stop_probe -- 127.0.0.1:25566 RustStop2
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustStop1 | RustStop2 |
| --- | ---: | ---: |
| player action packets sent | `2` | `2` |
| block changed ACK sequences | `[8, 9]` | `[8, 9]` |
| block destruction observations | `[]` | `[]` |
| block update | `{x:90,y:100,z:102,state_id:0}` | `{x:99,y:100,z:97,state_id:0}` |
| chunk packet | 54 | 54 |
| decoded chunk | 54 | 54 |
| chunk batch finished | 16 | 16 |
| received packet | 163 | 163 |

Server logundaki player konumları RustStop1 için `(90.5,101.0,102.5)`, RustStop2 için `(99.5,101.0,97.5)` idi; hedefler block update konumlarıyla sırasıyla `(90,100,102)` ve `(99,100,97)` oldu. Vanilla server console kontrolleri iki hedefin de stop sonrasında `air` olduğunu doğruladı. Actor oyuncuya destruction progress gönderilmemesi Java server kaynaklarındaki alıcı filtresiyle uyumludur. Vanilla logunda protocol error, timeout veya action warning oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server üzerinde survival start/stop action sırasını, stop eşiğini geçen dirt kırılmasını, iki ACK'i ve state `0` authoritative update'ini doğrular. Taş ve araç süreleri, tick bazlı progress, delayed destroy, survival item/tool etkileri, client prediction/rollback, ray hit ve actor dışı progress yayını açık kalır.

Sözleşme ayrıntıları [survival break stop contract belgesindedir](survival-break-stop-contract.md). Start→abort [ayrı kayıttadır](vanilla-survival-break-abort-parity.md); creative break ve placement [ayrı kayıtlardadır](vanilla-block-break-parity.md), [vanilla creative placement](vanilla-creative-placement-parity.md).
