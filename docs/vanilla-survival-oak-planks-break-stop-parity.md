# P12 — Vanilla 26.2 survival oak planks hardness doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü y=`100` oak planks platformu ve world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı.

```text
cargo run --offline --release --example survival_oak_planks_break_stop_probe -- 127.0.0.1:25566 RustOakStop1
cargo run --offline --release --example survival_oak_planks_break_stop_probe -- 127.0.0.1:25566 RustOakStop2
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustOakStop1 | RustOakStop2 |
| --- | ---: | ---: |
| player action packets sent | `2` | `2` |
| block changed ACK sequences | `[18,19]` | `[18,19]` |
| block destruction observations | `[]` | `[]` |
| block update observations | `[]` | `[]` |
| chunk packet | `54` | `54` |
| decoded chunk | `54` | `54` |
| chunk batch finished | `16` | `16` |
| received packet | `155` | `153` |

Server logundaki oyuncu konumları RustOakStop1 için `(109.5,101.0,95.5)`, RustOakStop2 için `(109.5,101.0,91.5)` idi; Java floor hedefleri sırasıyla `(109,100,95)` ve `(109,100,91)` oldu. Console `execute if block` kontrolleri iki hedefin de oak planks kaldığını doğruladı. Vanilla logunda final koşular için protocol error, timeout veya action warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç survival oak planks kısa start→stop akışında sequence ACK sırasını ve çıplak-el hardness eşiğinin aşılmadığını iki vanilla oturumunda doğrular. Diğer hardness/tool kombinasyonları, tick progress, delayed destroy, client prediction ve inventory etkileri açık kalır.

Sözleşme ayrıntıları [survival oak planks break stop contract belgesindedir](survival-oak-planks-break-stop-contract.md). Dirt'in tam kırılması [ayrı kayıttadır](vanilla-survival-break-stop-parity.md); stone hardness [ayrı kayıttadır](vanilla-survival-stone-break-stop-parity.md).
