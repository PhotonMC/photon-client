# P12 — Vanilla 26.2 survival delayed destroy doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü platform kısa süreliğine y=`100` dirt olacak şekilde ayarlanıp world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı.

```text
cargo run --offline --release --example survival_delayed_destroy_probe -- 127.0.0.1:25566 RustDelayed1
cargo run --offline --release --example survival_delayed_destroy_probe -- 127.0.0.1:25566 RustDelayed2
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustDelayed1 | RustDelayed2 |
| --- | ---: | ---: |
| player action packets sent | `2` | `2` |
| block changed ACK sequences | `[14,15]` | `[14,15]` |
| block destruction observations | `[]` | `[]` |
| delayed block update | `{x:105,y:100,z:92,state_id:0}` | `{x:93,y:100,z:96,state_id:0}` |
| chunk packet | `87` | `87` |
| decoded chunk | `87` | `87` |
| chunk batch finished | `27` | `27` |
| received packet | `218` | `293` |

Server logundaki oyuncu konumları RustDelayed1 için `(105.5,101.0,92.5)`, RustDelayed2 için `(93.5,101.0,96.5)` idi; Java floor hedefleri sırasıyla `(105,100,92)` ve `(93,100,96)` oldu. Console `execute if block` kontrolleri iki hedefin stop sonrasındaki delayed tick'lerde air olduğunu doğruladı. Vanilla logunda protocol error, timeout veya action warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç erken stop sonrasında server delayed destroy state'inin yeni action olmadan devam ettiğini iki vanilla oturumunda doğrular. Diğer hardness/tool kombinasyonları, delayed state iptali, hedef değiştirme, prediction ve görsel progress yayını açık kalır.

Sözleşme ayrıntıları [survival delayed destroy contract belgesindedir](survival-delayed-destroy-contract.md). Stone hardness [ayrı kayıttadır](vanilla-survival-stone-break-stop-parity.md); dirt tam stop [ayrı kayıttadır](vanilla-survival-break-stop-parity.md).
