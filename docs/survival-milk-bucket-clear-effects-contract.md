# P12 — Survival active multi-effect milk bucket cleanup sözleşmesi

Bu dilim, survival oyuncunun aktif poison, hunger ve regeneration effect'leri
varken milk bucket kullanmasını sınar. 32 tick sonunda milk bucket bucket'a
dönüşür ve üç effect'in tamamı server tarafından remove packet'iyle temizlenir.

## Java referansı

`Items.java:1254-1257` milk bucket'ı `CONSUMABLE` component'i ve bucket
remainder'ı ile kaydeder. `ClearAllStatusEffectsConsumeEffect.java:10-23`
aktif effect map'ini temizler. `LivingEntity.java:974-987,1105-1121` temizleme
sonucunu server tarafında effect koleksiyonuna uygular; `ServerPlayer.java:1654-1685`
her kayıt için update/remove packet'ini bağlantıya gönderir.

`ClientboundUpdateMobEffectPacket.java:50-63` effect başlangıcındaki entity,
registry ID, amplifier, duration ve flags alanlarını; remove packet'i
`ClientboundRemoveMobEffectPacket.java:14-20` entity ve effect ID'sini taşır.
Milk stack'inin `custom_model_data` patch'i component ID `17` olarak önceki
ItemStack parser kapsamından geçer.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Oyuncu bağlandıktan sonra console ile sırasıyla poison `60 0`, hunger `60 1`
  ve regeneration `60 0` effect'leri verilir; ardından şu stack slot 0'a konur:
  `minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `27`
- Kullanım sonrası drain: 2.5 saniye; bu süre 32 tick tüketimi ve tüm remove packet'lerini bekler

Beklenen invariant'lar carried/use packet sayılarının `1/1` olması, hazırlık
slotunun `1046/[17]`, bitiş slotunun `1040/[]`, ACK listesinin `[27]` olmasıdır.
Console ile verilen effect update'leri `18/0/1200/8 → 16/1/1200/8 → 9/0/1200/8`
sırasındadır. Milk tüketimi sonrası remove effect registry ID sırası
`16 → 18 → 9` olur; her remove aynı oyuncu entity ID'sini taşır.

## Sınır

Bu sözleşme milk bucket'ın clear-all effect yolunu, çoklu remove packet sırasını
ve bucket remainder'ını kapsar. Başka aktif effect kombinasyonları, diğer
component codec'leri, erken bırakma, iki el önceliği ve inventory click sonraki
dilimlerdir.
