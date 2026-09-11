# P12 — Survival spawn protection sözleşmesi

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
