# P12 — Vanilla 26.2 survival stone hardness doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü y=`100` stone platformu ve world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı.

```text
cargo run --offline --release --example survival_stone_break_stop_probe -- 127.0.0.1:25566 RustStoneStop1
cargo run --offline --release --example survival_stone_break_stop_probe -- 127.0.0.1:25566 RustStoneStop2
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustStoneStop1 | RustStoneStop2 |
| --- | ---: | ---: |
| player action packets sent | `2` | `2` |
| block changed ACK sequences | `[12,13]` | `[12,13]` |
| block destruction observations | `[]` | `[]` |
| block update observations | `[]` | `[]` |
| chunk packet | `54` | `54` |
| decoded chunk | `54` | `54` |
| chunk batch finished | `16` | `16` |
| received packet | `158` | `169` |

Server logundaki oyuncu konumları RustStoneStop1 için `(102.5,101.0,101.5)`, RustStoneStop2 için `(107.5,101.0,92.5)` idi; Java floor hedefleri sırasıyla `(102,100,101)` ve `(107,100,92)` oldu. Console `execute if block` kontrolleri iki hedefin de stone kaldığını doğruladı. Vanilla logunda protocol error, timeout veya action warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç survival stone kısa start→stop akışında sequence ACK sırasını ve çıplak-el hardness eşiğinin aşılmadığını iki vanilla oturumunda doğrular. Diğer hardness/tool kombinasyonları, tick progress, delayed destroy, client prediction ve inventory etkileri açık kalır.

Sözleşme ayrıntıları [survival stone break stop contract belgesindedir](survival-stone-break-stop-contract.md). Dirt'in tam kırılması [ayrı kayıttadır](vanilla-survival-break-stop-parity.md); start→abort [ayrı kayıttadır](vanilla-survival-break-abort-parity.md).
