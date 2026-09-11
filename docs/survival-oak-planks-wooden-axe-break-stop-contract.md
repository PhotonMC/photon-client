# P12 — Survival oak planks wooden axe tool progress sözleşmesi

Bu dilim, survival oyuncunun hotbar slot 0'daki wooden axe'ı seçip oak planks üzerinde `START_DESTROY_BLOCK` göndermesini, 1.2 saniye sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini sınar. Amaç wooden axe'ın doğru `mineable/axe` tool hızını oak planks sertliğiyle birlikte doğrulamak ve kırma sonrası dayanıklılık bileşeni yayınını gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

Java 26.2'de oak planks `strength(2.0F, 3.0F)` ile kayıtlıdır. Wooden axe `ToolMaterial.WOOD` kullanır; wooden material'ın mining speed'i `2.0F`, dayanıklılığı `59` ve axe kaydı `Items.java` içindedir. Tool material, eşleşen `mineable/axe` blokları için `TOOL` component'ine bu hızı ve blok başına `1` hasarı yazar; aynı item `WEAPON` component'ini de taşır.

`BlockBehaviour` doğru tool için destroy progress modifier'ını `30` seçer. Server game mode, geçen tick sayısıyla progress'i çarpar ve `0.7F` stop eşiği geçilince hedefi kırar. Kırma dayanıklılık değişimi, wooden axe slotuna `DAMAGE` component patch'i olarak yayınlanır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:wooden_axe`
- Rust hazırlığı: on iki saniyelik console komutu aralığından sonra `SET_CARRIED_ITEM`, slot `0`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` oak planks platformu
- Direction: `UP`, ID `1`
- Start sequence: `32`
- Stop sequence: `33`
- Start ile stop arasında bekleme: `1_200 ms`

Rust probe `SET_CARRIED_ITEM` ve iki action paketini yollar; ACK, slot ve block update listelerini toplar. Beklenen sonuç carried item packet sayısı `1`, action sayısı `2`, ACK `[32,33]`, hazırlık slotunda item registry `942` ve boş patch, bitiş slotunda aynı item için `DAMAGE` component ID `3`, ayrıca hedefte state `0` block update'idir.

## Sınır

Bu dilim yalnızca wooden axe ile oak planks kırmayı ve tek blok dayanıklılık güncellemesini sınar. Farklı tool/block kombinasyonları, tool dayanıklılığının birden fazla blokta ilerlemesi, enchantment, erken stop, tick başına destruction progress, client prediction ve inventory click sonraki dilimlerdir.
