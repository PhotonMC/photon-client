# P12 — Vanilla 26.2 survival spawn protection doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve `spawn-protection=16` ayarlarıyla çalıştırıldı. Kontrollü y=`100` dirt platformu ve world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı. Ops listesinde `spawnadmin` bulundu; probe oyuncuları op değildi.

```text
cargo run --offline --release --example survival_spawn_protection_probe -- 127.0.0.1:25566 RustSpawnP12
cargo run --offline --release --example survival_spawn_protection_probe -- 127.0.0.1:25566 RustSpawnP13
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustSpawnP12 | RustSpawnP13 |
| --- | ---: | ---: |
| player action packets sent | `1` | `1` |
| block changed ACK sequences | `[16]` | `[16]` |
| system chat observations | `overlay=true`, `build.spawn_protection` | `overlay=true`, `build.spawn_protection` |
| block destruction observations | `[]` | `[]` |
| block update observations | `[]` | `[]` |
| chunk packet | `24` | `24` |
| decoded chunk | `24` | `24` |
| chunk batch finished | `6` | `6` |
| received packet | `103` | `103` |

Server logundaki oyuncu konumları RustSpawnP12 için `(101.5,101.0,108.5)`, RustSpawnP13 için `(97.5,101.0,106.5)` idi; Java floor hedefleri sırasıyla `(101,100,108)` ve `(97,100,106)` oldu. Console `execute if block` kontrolleri iki hedefin de dirt kaldığını doğruladı. Hedeflerin world spawn'a Chebyshev uzaklığı sırasıyla `8` ve `6` bloktu; ikisi de `16` yarıçapının içindeydi. Vanilla logunda final koşular için protocol error, timeout veya action warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç korumalı spawn alanındaki op olmayan survival oyuncunun block-break isteğinin dünya değişikliği olmadan reddedildiğini, action sequence'inin yine ACK edildiğini ve kırmızı overlay sistem sohbetiyle `build.spawn_protection` component'inin gönderildiğini iki vanilla oturumunda doğrular. Spawn dışı yarıçap, op oyuncu, boş ops listesi, sıfır/negatif koruma ve başka respawn dimension sonraki senaryolardır.

Sözleşme ayrıntıları [survival spawn protection contract belgesindedir](survival-spawn-protection-contract.md). Menzil reddi [ayrı kayıttadır](vanilla-block-break-rejection-parity.md); survival abort, stop, stone hardness ve delayed destroy sonuçları ilgili parity belgelerinde tutulur.
