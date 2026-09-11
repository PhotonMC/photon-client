# P12 — Survival stone hardness sözleşmesi

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
