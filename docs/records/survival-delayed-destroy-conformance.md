# P12 — Survival delayed destroy sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun dirt kırmayı başlatıp eşik altında erken durdurmasından sonra vanilla server'ın delayed destroy state'ini sonraki tick'lerde tamamlamasını sınar. Amaç `STOP` sonrasında yeni client action gönderilmeden authoritative block update oluştuğunu doğrulamaktır.

## Java referansı

Java `ServerPlayerGameMode`, aynı hedefteki erken `STOP` sırasında `hasDelayedDestroy` bayrağını kurar ve başlangıç tick'ini `delayedTickStart` olarak saklar. Sonraki `tick()` çağrıları bu başlangıçtan itibaren progress hesaplar; progress `1.0F` olduğunda `destroyBlock` çalışır ve hedef air olur. Action sequence'leri bağlantı katmanında ACK edilir.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` dirt platformu
- Direction: `UP`, ID `1`
- Start sequence: `14`
- Stop sequence: `15`
- Start ile stop arasında bekleme: `100 ms`
- Stop sonrası bağlantı drain'i: `1.2 s`

Rust probe yalnızca start ve stop action paketlerini yollar; stop sonrasında açık bağlantıda server paketlerini yaklaşık 1.2 s toplar. Beklenen sonuç ACK `[14,15]`, yeni action gönderilmemesi, actor için destruction progress alınmaması ve delayed kırılmanın hedef state `0` block update'iyle görünmesidir.

## Sınır

Bu dilim delayed state'in tek hedefte tamamlanmasını doğrular. Erken abort, başka hedefe geçiş, delayed state iptali, araç/hardness kombinasyonları, tick başına progress ve client prediction sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü platform kısa süreliğine y=`100` dirt olacak şekilde ayarlanıp world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı.

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

Sözleşme ayrıntıları [survival delayed destroy contract belgesindedir](#sözleşme). Stone hardness [ayrı kayıttadır](survival-stone-break-stop-conformance.md); dirt tam stop [ayrı kayıttadır](survival-break-stop-conformance.md).
