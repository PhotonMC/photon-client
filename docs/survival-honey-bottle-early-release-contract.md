# P12 — Survival honey bottle erken bırakma sözleşmesi

Bu dilim, survival oyuncunun component-bearing honey bottle kullanımını başlatıp 40 tick'lik tüketim süresi dolmadan bırakmasını sınar. Amaç `RELEASE_USE_ITEM` action'ının kullanım durumunu kapatmasını, honey bottle'ı tüketmemesini ve yalnızca kullanım başlangıcındaki stack'in korunmasını gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

`Items.java:1930-1933` honey bottle'ı 2.0 saniyelik `Consumables.HONEY_BOTTLE` ile kaydeder. `Consumables.java:16-20` bu süreyi 40 tick yapar. `MultiPlayerGameMode.java:514-518` bırakmada `RELEASE_USE_ITEM`, `BlockPos.ZERO`, `Direction.DOWN` ve varsayılan sequence `0` gönderir; ardından yerel kullanım durumunu bırakır.

Server listener `RELEASE_USE_ITEM` action'ında `LivingEntity.releaseUsingItem` çağırıp action handler'ından ACK yoluna girmeden döner. `LivingEntity.java:3611-3622`, item'in `releaseUsing` davranışını çalıştırıp kullanım state'ini kapatır; honey bottle'ın temel `Item.releaseUsing` yolu başarı döndürmediği için `completeUsingItem` çağrılmaz ve stack tüketilmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: slot 0'a `minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `34`
- Erken bırakma: 250 ms sonra `RELEASE_USE_ITEM`, action ordinal `5`, `BlockPos.ZERO`, `DOWN`, sequence `0`
- Tüketim süresi: 40 tick; bırakma bu süreden önce yapılır

Beklenen invariant'lar carried item packet sayısı `1`, use packet sayısı `1`, player action packet sayısı `1`, ACK `[34]`; hazırlık slot update'i count `1`, honey bottle item ID `1412` ve pozitif component `[17]`; kullanım sonrasında slot/content dönüşümü bulunmamasıdır. `RELEASE_USE_ITEM` için ayrı block-change ACK'i beklenmez. İlk content update'i state `1` ve 46 slot taşır.

## Sınır

Bu sözleşme honey bottle'ın erken bırakılmasını ve component-bearing stack'in korunmasını kapsar. Tam tüketim, aktif poison temizleme, başka `useOnRelease` item'leri, iki el önceliği, kullanım sırasında saldırı ve inventory click sonraki dilimlerdir.
