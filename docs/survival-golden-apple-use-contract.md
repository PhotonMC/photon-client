# P12 — Survival golden apple use sözleşmesi

Bu dilim, survival oyuncunun golden apple'ı tüketmesini, 32 tick'lik food
kullanımını, stack tüketimini ve golden apple'a bağlı iki status effect'in wire
sırasını sınar. Golden apple vanilla food tanımında her zaman yenebilir ve
item stack'i pozitif component patch'i taşımaz.

## Java referansı

`Items.java:1124-1129` golden apple'ı `Foods.GOLDEN_APPLE` ve
`Consumables.GOLDEN_APPLE` ile kaydeder. `Foods.java:24` food'un her zaman
yenebilir olduğunu, `Consumables.java:38-44` ise tüketim sonunda regeneration
ve absorption effect'lerini bu sırayla eklediğini gösterir. Default food
consumable süresi `Consumables.java:68-74` içinde 1.6 saniye, yani 32 tick'tir.

`LivingEntity.java:3480-3484,3574-3587` kullanım süresi bitince stack'i
tamamlar. `ApplyStatusEffectsConsumeEffect.java:15-63` effect listesini
sıralı olarak uygular. ItemStack wire codec'i `ItemStack.java:122-200`
count, item registry ID'si ve component patch listesini taşır; vanilla golden
apple hazırlık stack'inde patch listesi boştur.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:golden_apple`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `36`
- Kullanım sonrası drain: 2.5 saniye; bu süre 32 server tick'inin tamamlanmasını
  ve effect/slot yayınlarını bekler

Beklenen invariant'lar carried/use packet sayılarının `1/1` olması, hazırlık
slotunun state `2`, slot `36`, count `1`, item ID `1014`, component `[]`
taşıması, başlangıç content'inin state `1` ve 46 slot olması, bitiş slotunun
boş yayınlanması ve ACK listesinin `[36]` olmasıdır. Effect update'leri aynı
oyuncu entity ID'siyle şu sırayı korur: regeneration `9`, amplifier `1`,
duration `100`, flags `14`; absorption `21`, amplifier `0`, duration `2400`,
flags `14`. Bu akışta remove effect paketi beklenmez.

## Sınır

Bu sözleşme golden apple consumable akışını, patch'siz item stack'i ve iki
effect update sırasını kapsar. Enchanted golden apple, diğer consumable/effect
varyantları, effect yenileme, iki el önceliği ve inventory click sonraki
dilimlerdir.
