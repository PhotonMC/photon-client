# P12 — Survival stone hardness sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun çıplak elle stone üzerinde `START_DESTROY_BLOCK` gönderip 650 ms sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini sınar. Amaç dirt'in önceki tam kırılma eşiğinden farklı olarak stone'un kısa sürede kırılmamasını, sequence ACK'lerini ve dünya state'inin korunmasını gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

`ServerPlayerGameMode` stop akışında geçen tick sayısını block destroy progress'i ile çarpar ve `0.7F` eşiğini aşınca block'u kırar. Stone'un çıplak el destroy speed'i bu 650 ms aralığında eşiği geçmez; server action'ları ACK kuyruğuna alır fakat hedef için state değişikliği üretmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` stone platformu
- Direction: `UP`, ID `1`
- Start sequence: `12`
- Stop sequence: `13`
- Start ile stop arasında bekleme: `650 ms`

Rust probe iki action paketini yollar ve ACK, destruction progress ve block update listelerini toplar. Beklenen sonuç ACK `[12,13]`, boş destruction listesi, boş block update listesi ve server tarafında stone hedefin korunmasıdır.

## Sınır

Bu dilim yalnızca kısa çıplak-el stone eşiğini doğrular. Tick başına progress, araç/enchanted tool süreleri, diğer block hardness değerleri, delayed destroy, client prediction ve başka oyunculara progress yayını sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü y=`100` stone platformu ve world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı.

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

Sözleşme ayrıntıları [survival stone break stop contract belgesindedir](#sözleşme). Dirt'in tam kırılması [ayrı kayıttadır](survival-break-stop-conformance.md); start→abort [ayrı kayıttadır](survival-break-abort-conformance.md).
