# P12 — Survival delayed destroy sözleşmesi

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
