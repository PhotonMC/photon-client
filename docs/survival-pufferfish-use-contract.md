# P12 — Survival food component override'lı pufferfish use sözleşmesi

Bu dilim, survival oyuncunun pufferfish'i tüketmesini, 32 tick'lik food
kullanımını, stack tüketimini ve pufferfish'e bağlı üç status effect'in wire
sırasını sınar. Tam açlıkta vanilla pufferfish kullanılamadığı için test
stack'i Java sunucusunun `food` component override'ı ile `can_always_eat=true`
yapılır; item türü ve tüketim effect'i yine vanilla pufferfish olarak kalır.

## Java referansı

`Items.java:1321-1325` pufferfish'i `Foods.PUFFERFISH` ve
`Consumables.PUFFERFISH` ile kaydeder. `Foods.java:33` default food'un tam
açlıkta yenemediğini, `Consumables.java:48-58` ise tüketim sonunda poison,
hunger ve nausea effect'lerini bu sırayla eklediğini gösterir. Default food
consumable süresi `Consumables.java:68-74` içinde 1.6 saniye, yani 32 tick'tir.

`FoodProperties.java:20-37` food component'inin nutrition, saturation ve
`can_always_eat` alanlarını VarInt, f32 ve boolean olarak taşır. Pinned
`DataComponents` sırasındaki food component ID'si `23`'tür. ItemStack patch'i
`ItemStack.java:122-200` içindeki count, item registry ID'si ve component
listelerinden oluşur; istemci parser'ı bu component gövdesini tüketir.

`LivingEntity.java:3480-3484,3574-3587` kullanım süresi bitince stack'i
tamamlar. `ApplyStatusEffectsConsumeEffect.java:15-63` effect listesini
sıralı olarak uygular. Clientbound update effect paketi entity ID, effect
registry ID, amplifier, duration ve flags alanlarını taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:pufferfish[food={nutrition:1,saturation:0.1,can_always_eat:true}]`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `35`
- Kullanım sonrası drain: 2.5 saniye; bu süre 32 server tick'inin tamamlanmasını
  ve effect/slot yayınlarını bekler

Beklenen invariant'lar carried/use packet sayılarının `1/1` olması, hazırlık
slotunun state `2`, slot `36`, count `1`, item ID `1089`, component `[23]`
taşıması, başlangıç content'inin state `1` ve 46 slot olması, bitiş slotunun
boş yayınlanması ve ACK listesinin `[35]` olmasıdır. Effect update'leri aynı
oyuncu entity ID'siyle şu sırayı korur: poison `18`, amplifier `1`, duration
`1200`, flags `14`; hunger `16`, amplifier `2`, duration `300`, flags `14`;
nausea `8`, amplifier `0`, duration `300`, flags `14`. Bu akışta remove effect
paketi beklenmez.

## Sınır

Bu sözleşme food component codec'ini ve pufferfish consumable effect sırasını
kapsar. Default pufferfish'in tam açlık reddi [ayrı negative contract'ta](survival-pufferfish-full-hunger-rejection-contract.md)
doğrulanır. Daha fazla component codec'i, diğer consumable/effect varyantları,
effect yenileme, iki el önceliği ve inventory click sonraki dilimlerdir.
