# P12 — Survival spawn protection sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun world spawn çevresindeki korumalı bir bloğu kırmayı denediğinde vanilla server'ın dünya değişikliği yapmadan reddetme mesajı ve action sequence ACK'i üretmesini sınar.

## Java referansı

`ServerPlayerGameMode`, `START_DESTROY_BLOCK` için oyuncunun op durumunu, ops listesini, respawn dimension'ını, spawn protection yarıçapını ve hedefin spawn'a Chebyshev uzaklığını kontrol eder. Koruma etkinse block-break işlemi erken döner ve oyuncuya `build.spawn_protection` translatable component'i kırmızı overlay olarak gönderilir. Packet listener action sequence'ini işleme sonrasında ACK kuyruğuna alır.

`ClientboundSystemChatPacket` component'i ve overlay boolean'ını taşır. Bu probe component'in `translate` alanını ve overlay değerini NBT payload'ından çözer.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Server ayarı: `spawn-protection=16`
- Ops listesi: en az bir başka op oyuncu bulunur; probe oyuncusu op değildir
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- World spawn: kontrollü y=`100` dirt platformu üzerindeki `(100,101,100)`
- Action: `START_DESTROY_BLOCK`, direction `UP` (ID `1`), sequence `16`

Rust probe yalnızca bu action paketini yollar ve ilk dünya yüklemesinden sonra kısa bir drain yapar. Beklenen sonuç sequence ACK `[16]`, hedefe ait block update veya destruction progress olmaması, `translation_key=Some("build.spawn_protection")` ve `overlay=true` değeridir.

## Sınır

Bu sözleşme korunan spawn alanındaki op olmayan survival oyuncuyu kapsar. Spawn dışı hedefler, op oyuncu, boş ops listesi, sıfır/negatif koruma ve başka respawn dimension sonraki senaryolardır.


## Vanilla server-conformance doğrulaması

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

Sözleşme ayrıntıları [survival spawn protection contract belgesindedir](#sözleşme). Menzil reddi [ayrı kayıttadır](block-break-rejection-conformance.md); survival abort, stop, stone hardness ve delayed destroy sonuçları ilgili birleşik kayıtlarda tutulur.
