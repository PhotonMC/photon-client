# P12 — Survival component-bearing milk bucket use sözleşmesi

Bu dilim, survival oyuncunun slot 0'daki süt kovasını kullanmasını ve 32 tick'lik
içme süresinin sonunda kovaya dönüşmesini sınar. Stack ayrıca `custom_model_data`
component patch'i taşır; amaç item stack wire codec'inde pozitif component'in
tüketim öncesi okunması ve dönüşen remainder stack'inin patch'siz gözlenmesidir.

## Java referansı

`Items.java:1254-1257` süt kovasını tekli stack, `Consumables.MILK_BUCKET` ve
`usingConvertsTo(BUCKET)` ile kaydeder. `Consumables.java:65,72-74` süt kovasının
1.6 saniyelik (32 tick) drink consumable olduğunu ve consume efektlerinin
durum efektlerini temizlediğini belirler. Genel item yolu `Item.java:189-216`
component'i bulup kullanımı başlatır ve tamamlandığında component'in
`onConsume` yolunu çalıştırır; `LivingEntity.java:3480-3484,3574-3587` server
tick'inde sayacı azaltıp tamamlanan stack'i eldeki item'a yazar.

`ItemStack.java:122-200` optional stack wire codec'inde count, item registry
ID'si ve component patch count/listelerini taşır. `DataComponents.java:154-156`
ile kayıtlı `custom_model_data` component'i bu koşuda pozitif component ID `17`
olarak gelir. Rust probe bu component'in dört listesini tüketir ve pozitif/
negatif component ID'lerini container gözlemine yazar; diğer pozitif component
codec'leri henüz bu dar dilimin dışındadır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: slot 0'a
  `minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `27`
- Kullanım sonrası drain: 2.5 saniye; bu süre 32 server tick'inin tamamlanmasını
  ve inventory dönüşümünün yayınlanmasını bekler

Beklenen invariant'lar şunlardır: carried item packet sayısı `1`, use packet
sayısı `1`, `use_item_on` sayısı `0`, ACK `[27]`; hazırlık slot update'i
count `1`, milk bucket item ID `1046` ve pozitif component `[17]`; bitiş slot
update'i state `3`, slot `36`, count `1`, bucket item ID `1040` ve boş component
listeleridir. İlk content update'i state `1` ve 46 slot taşır.

## Sınır

Bu sözleşme milk bucket'ın yerleşik consumable akışını, component-bearing stack
codec'ini ve bucket remainder dönüşümünü kapsar. Component değerlerini oyun
renderer'ında kullanmak, diğer component türlerini decode etmek, status effect
uygulama/temizleme sonucu, farklı consumable'lar, erken bırakma, iki el önceliği,
inventory click ve client prediction sonraki dilimlerdir.
