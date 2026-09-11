# P12 — Survival dirt wooden shovel tool progress sözleşmesi

Bu dilim, survival oyuncunun hotbar slot 0'daki wooden shovel'ı seçip dirt üzerinde `START_DESTROY_BLOCK` göndermesini, 650 ms sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini sınar. Amaç çıplak el dirt akışından farklı olarak wooden shovel'ın tool hızının kırma eşiğini geçirmesini ve kırma sonrası dayanıklılık bileşeni yayınını gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

Java 26.2'de dirt `strength(0.5F)` ile kayıtlıdır. Wooden shovel `ToolMaterial.WOOD` kullanır; wooden material'ın mining speed'i `2.0F`, dayanıklılığı `59` ve `ShovelItem` kaydı `Items.java` içindedir. Tool material, eşleşen `mineable/shovel` blokları için `TOOL` component'ine bu hızı ve blok başına `1` hasarı yazar; aynı item `WEAPON` component'ini de taşır.

`BlockBehaviour` doğru tool için destroy progress modifier'ını `30` seçer. Server game mode, geçen tick sayısıyla progress'i çarpar ve `0.7F` stop eşiği geçilince hedefi kırar. Kırma dayanıklılık değişimi, wooden shovel slotuna `DAMAGE` component patch'i olarak yayınlanır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:wooden_shovel`
- Rust hazırlığı: on iki saniyelik console komutu aralığından sonra `SET_CARRIED_ITEM`, slot `0`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` dirt platformu
- Direction: `UP`, ID `1`
- Start sequence: `30`
- Stop sequence: `31`
- Start ile stop arasında bekleme: `650 ms`

Rust probe `SET_CARRIED_ITEM` ve iki action paketini yollar; ACK, slot ve block update listelerini toplar. Beklenen sonuç carried item packet sayısı `1`, action sayısı `2`, ACK `[30,31]`, hazırlık slotunda item registry `940` ve boş patch, bitiş slotunda aynı item için `DAMAGE` component ID `3`, ayrıca hedefte state `0` block update'idir.

## Sınır

Bu dilim yalnızca wooden shovel ile dirt kırmayı ve tek blok dayanıklılık güncellemesini sınar. Farklı tool/block kombinasyonları, tool dayanıklılığının birden fazla blokta ilerlemesi, enchantment, erken stop, tick başına destruction progress, client prediction ve inventory click sonraki dilimlerdir.
