# P12 — Survival honey bottle poison temizleme sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her oyuncu oyuna girdikten
sonra item ve effect console komutları başarı satırları görülerek uygulandı.

```text
cargo run --offline --release --example survival_honey_bottle_poison_cleanup_probe -- 127.0.0.1:25566 RustHoneyPoison1
cargo run --offline --release --example survival_honey_bottle_poison_cleanup_probe -- 127.0.0.1:25566 RustHoneyPoison2

item replace entity <player> hotbar.0 with minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]
effect give <player> minecraft:poison 60 0 true
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustHoneyPoison1 | RustHoneyPoison2 |
| --- | ---: | ---: |
| player entity ID | `873` | `918` |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| mob-effect update | `entity=873, effect=18, amplifier=0, duration=1200, flags=8` | `entity=918, effect=18, amplifier=0, duration=1200, flags=8` |
| mob-effect removal | `entity=873, effect=18` | `entity=918, effect=18` |
| container slot preparation | `state=2, slot=36, count=1, item=1412, components=[17]` | `state=2, slot=36, count=1, item=1412, components=[17]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container slot after use | `state=3, slot=36, count=1, item=1149, components=[]` | `state=3, slot=36, count=1, item=1149, components=[]` |
| block changed ACK sequences | `[29]` | `[29]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `3571` | `2693` |

`use_item_on` gönderilmedi; dolu honey bottle doğrudan `use_item` yolundan
kullanıldı. Poison effect update'i kullanım öncesinde, remove paketi 40 tick'lik
tüketim tamamlandıktan sonra geldi. Her iki koşuda da glass bottle remainder'ı
aynı container slot update'iyle yayınlandı. Server logunda item/effect komutları,
normal disconnect ve protocol error/warning olmaması gözlendi.

## Sonuç ve açık kapsam

İki vanilla oturumu, Java'nın aktif poison temizleme davranışını, effect update ve
remove packet alanlarını, 40 tick tüketim süresini, sequence ACK'ini, component
patch'li hazırlık stack'ini ve glass bottle remainder'ını aynı invariant'larla
verdi. Diğer effect türleri, erken bırakma, diğer consumable sonuçları, diğer
component codec'leri, inventory click ve client prediction açık kalır.

Sözleşme ayrıntıları [survival honey bottle poison cleanup contract belgesindedir](#sözleşme).
