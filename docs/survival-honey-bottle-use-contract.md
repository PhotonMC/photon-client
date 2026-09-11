# P12 — Survival component-bearing honey bottle use sözleşmesi

Bu dilim, survival oyuncunun slot 0'daki honey bottle'ı kullanmasını ve 40
tick'lik içme süresinin sonunda glass bottle'a dönüşmesini sınar. Stack ayrıca
`custom_model_data` component patch'i taşır; aynı container wire parser'ı
tüketim öncesi pozitif component'i ve tüketim sonrası boş patch'i gözlemler.

## Java referansı

`Items.java:1930-1933` honey bottle'ı `FOOD` ve `Consumables.HONEY_BOTTLE`,
`usingConvertsTo(GLASS_BOTTLE)` ve maksimum stack 16 ile kaydeder.
`Consumables.java:16-20` tüketim süresini 2.0 saniye (40 tick), animasyonu drink
ve honey drink sesini seçer. Genel `Item.java:189-216` consumable component'i
kullanımı başlatır; `LivingEntity.java:3480-3484,3574-3587` server tick'leri
sonunda consume ve remainder dönüşümünü tamamlar.

`ItemStack.java:122-200` optional stack wire codec'inde count, item registry
ID'si ve component patch listelerini taşır. Bu probe `custom_model_data`
component ID `17` için doğrulanan dört liste codec'ini kullanır; başka pozitif
component türleri bu dar parser kapsamına dahil değildir.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: slot 0'a
  `minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `28`
- Kullanım sonrası drain: 2.5 saniye; bu süre 40 server tick'inin tamamlanmasını
  ve glass bottle dönüşümünün yayınlanmasını bekler

Beklenen invariant'lar şunlardır: carried item packet sayısı `1`, use packet
sayısı `1`, `use_item_on` sayısı `0`, ACK `[28]`; hazırlık slot update'i
count `1`, honey bottle item ID `1412` ve pozitif component `[17]`; bitiş slot
update'i state `3`, slot `36`, count `1`, glass bottle item ID `1149` ve boş
component listeleridir. İlk content update'i state `1` ve 46 slot taşır.

## Sınır

Bu sözleşme honey bottle consumable akışını, component-bearing stack codec'ini
ve glass bottle remainder dönüşümünü kapsar. Poison temizleme efektinin aktif
bir effect üzerindeki sonucu, diğer component codec'leri, diğer consumable'lar,
erken bırakma, iki el önceliği, inventory click ve client prediction sonraki
dilimlerdir.
