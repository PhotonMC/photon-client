# P12 — Survival honey bottle poison temizleme sözleşmesi

Bu dilim, survival oyuncunun aktif poison effect varken component-bearing honey
bottle kullanmasını sınar. Kullanım tamamlandığında honey bottle glass bottle'a
dönüşmeli ve yalnızca poison effect'i kaldıran clientbound effect paketi
gözlenmelidir.

## Java referansı

`Items.java:1930-1933` honey bottle'ı `Consumables.HONEY_BOTTLE` ve
`usingConvertsTo(GLASS_BOTTLE)` ile kaydeder. `Consumables.java:16-20` içme
süresini 2.0 saniye (40 tick) yapar ve tüketim sonunda
`RemoveStatusEffectsConsumeEffect(MobEffects.POISON)` çalıştırır.

`RemoveStatusEffectsConsumeEffect.java:20-42` effect holder'ı
`LivingEntity.removeEffect` ile kaldırır. `LivingEntity.java:1065-1072` ve
`ServerPlayer.java:1672-1682` kaldırmayı tamamladığında oyuncuya
`ClientboundRemoveMobEffectPacket` yollar. Effect hazırlığı
`ServerPlayer.java:1657-1669` üzerinden `ClientboundUpdateMobEffectPacket` ile
yayınlanır. Bu packet'lerin 26.2 play dispatch ID'leri sırasıyla `78` ve
`132`'dir; update alanları entity ID, effect registry ID, amplifier, duration,
flags; remove alanları entity ID ve effect registry ID'dir.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı:
  `item replace entity <player> hotbar.0 with minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]`
- Aktif effect hazırlığı: `effect give <player> minecraft:poison 60 0 true`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `29`
- Kullanım sonrası drain: 2.5 saniye; 40 tick tüketim ve effect kaldırma yayını
  bu aralıkta beklenir

Beklenen invariant'lar şunlardır: carried item packet sayısı `1`, use packet
sayısı `1`, `use_item_on` sayısı `0`, ACK `[29]`; effect update listesinde
oyuncunun entity ID'si, poison registry ID `18`, amplifier `0`, yakalama komutu
nedeniyle duration `1200` ve flags `8`; effect remove listesinde aynı entity ve
effect ID; hazırlık slot update'i `item=1412`, pozitif component `[17]`; bitiş
slot update'i state `3`, slot `36`, glass bottle `item=1149` ve boş component
listeleridir. İlk content update'i state `1` ve 46 slot taşır.

## Sınır

Bu sözleşme honey bottle'ın aktif poison'u temizlemesini, effect packet wire
alanlarını ve glass bottle remainder dönüşümünü kapsar. Diğer effect türleri,
erken bırakma, diğer consumable sonuçları, diğer component codec'leri, inventory
click ve client prediction sonraki dilimlerdir.
