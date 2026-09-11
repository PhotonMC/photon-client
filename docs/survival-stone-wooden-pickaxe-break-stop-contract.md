# P12 — Survival wooden pickaxe tool progress sözleşmesi

Bu dilim, survival oyuncunun hotbar slot 0'daki wooden pickaxe'ı seçip stone üzerinde `START_DESTROY_BLOCK` göndermesini, yeterli tick geçtikten sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini sınar. Amaç çıplak el stone probe'undan farklı olarak doğru tool'un destroy progress hesabına etkisini gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

Java 26.2'de stone `strength(1.5F, 6.0F)` ve wooden pickaxe `ToolMaterial.WOOD` üzerinden pickaxe özellikleriyle kayıtlıdır. `BlockBehaviour` doğru tool için modifier `30` kullanır ve oyuncunun tool destroy speed'ini stone destroy speed'ine böler. Server game mode progress'i `0.7F` eşiğine ulaştığında hedefi kırar ve state `0` block update'i yollar.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:wooden_pickaxe`
- Rust hazırlığı: on iki saniyelik console komutu aralığından sonra `SET_CARRIED_ITEM`, slot `0`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` stone platformu
- Direction: `UP`, ID `1`
- Start sequence: `20`
- Stop sequence: `21`
- Start ile stop arasında bekleme: `2200 ms`

Rust probe `SET_CARRIED_ITEM` ve iki action paketini yollar; ACK, destruction progress ve block update listelerini toplar. Beklenen sonuç carried item packet sayısı `1`, ACK `[20,21]`, destruction listesi boş ve hedefte state `0` block update'idir.

## Sınır

Bu dilim yalnızca server tarafından hotbar'a yerleştirilen wooden pickaxe ile stone kırma süresini sınar. Tool dayanıklılığı, enchantment, farklı tool/block kombinasyonları, client prediction, inventory slot yayınları ve tick başına progress sonraki dilimlerdir.
