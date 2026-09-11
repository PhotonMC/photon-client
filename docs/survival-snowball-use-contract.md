# P12 — Survival snowball tüketimi sözleşmesi

Bu dilim, survival oyuncunun slot 0'daki tek snowball stack'ini seçip ana elle `use_item` göndermesini sınar. Amaç dolu stack item kullanımının action sequence ACK'ini, projectile spawn'ını ve tüketim sonrası clientbound container slot düzeltmesini gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

`ServerGamePacketListenerImpl` `use_item` action'ında sequence'i ACK kuyruğuna alır ve eldeki boş olmayan stack'i game mode'a verir. `SnowballItem.use` server level'da rotation'dan projectile üretir, kullanım istatistiğini günceller ve stack'ten bir item tüketir. Tüketilen player inventory hotbar slotu `ClientboundContainerSetSlotPacket` ile boş stack olarak gönderilir. Item stack wire codec'i boş stack için count `0` taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:snowball`
- Rust hazırlığı: on iki saniyelik console komutu aralığından sonra `SET_CARRIED_ITEM`, slot `0`
- Action: `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `22`
- Stack: slot `0`'da tek snowball

Rust probe container slot paketlerini, add entity paketlerini ve ACK'leri toplar. Beklenen sonuç carried item packet sayısı `1`, use item packet sayısı `1`, ACK `[22]`, snowball entity type `120` ile bir add-entity gözlemi ve hotbar slot `36` için boş optional item stack (`item_count=None`) gözlemidir. Hazırlık sırasında slot `36` için count `1` stack güncellemesi de görülebilir.

## Sınır

Bu dilim tek stack snowball tüketimini ve projectile spawn wire akışını kapsar. Item component patch'leri, farklı stack count'ları, projectile hareket/impact, iki el önceliği, inventory click ve client prediction sonraki dilimlerdir.
