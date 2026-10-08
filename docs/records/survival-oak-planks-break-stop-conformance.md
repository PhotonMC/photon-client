# P12 — Survival oak planks hardness sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun çıplak elle oak planks üzerinde `START_DESTROY_BLOCK` gönderip 650 ms sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini sınar. Amaç, dirt'in önceki tam kırılma sonucundan ve stone'un kısa hardness sonucundan farklı bir blok dayanımının aynı action/ACK akışında korunmasını gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

Java 26.2'de `Blocks.OAK_PLANKS` destroy time değeri `2.0F` olarak tanımlıdır. `BlockBehaviour` çıplak el için destroy speed'i destroy time ve `100` modifier'ına böler; `ServerPlayerGameMode` geçen tick'lerle progress'i biriktirir ve `0.7F` eşiği aşılmadığında stop action'ı bloğu kırmaz. Bu 650 ms koşulunda oak planks hedefinin state'i korunmalıdır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` oak planks platformu
- Direction: `UP`, ID `1`
- Start sequence: `18`
- Stop sequence: `19`
- Start ile stop arasında bekleme: `650 ms`

Rust probe iki action paketini yollar ve ACK, destruction progress ve block update listelerini toplar. Beklenen sonuç ACK `[18,19]`, boş destruction listesi, boş block update listesi ve server tarafında oak planks hedefin korunmasıdır.

## Sınır

Bu dilim yalnızca kısa çıplak-el oak planks eşiğini doğrular. Her tick progress'i, araç/enchanted tool süreleri, diğer block hardness değerleri, delayed destroy, client prediction ve başka oyunculara progress yayını sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü y=`100` oak planks platformu ve world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı.

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

Sözleşme ayrıntıları [survival oak planks break stop contract belgesindedir](#sözleşme). Dirt'in tam kırılması [ayrı kayıttadır](survival-break-stop-conformance.md); stone hardness [ayrı kayıttadır](survival-stone-break-stop-conformance.md).
