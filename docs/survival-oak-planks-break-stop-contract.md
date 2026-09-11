# P12 — Survival oak planks hardness sözleşmesi

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
