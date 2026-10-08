# P12 — Survival food component override'lı pufferfish use sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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
kapsar. Default pufferfish'in tam açlık reddi [ayrı negative contract'ta](survival-pufferfish-full-hunger-rejection-conformance.md)
doğrulanır. Daha fazla component codec'i, diğer consumable/effect varyantları,
effect yenileme, iki el önceliği ve inventory click sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Vanilla pufferfish tam
açlıkta kullanılamadığı için her oyuncu bağlandıktan sonra item'in food
component'i sunucu console'undan override edildi. Console başarı satırı
görülmeden koşu geçerli sayılmadı.

```text
cargo run --offline --release --example survival_pufferfish_use_probe -- 127.0.0.1:25566 RustPufferH4
cargo run --offline --release --example survival_pufferfish_use_probe -- 127.0.0.1:25566 RustPufferH5

item replace entity <player> hotbar.0 with minecraft:pufferfish[food={nutrition:1,saturation:0.1,can_always_eat:true}]
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustPufferH4 | RustPufferH5 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1089, components=[23]` | `state=2, slot=36, count=1, item=1089, components=[23]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container slot after use | `state=3, slot=36, empty` | `state=3, slot=36, empty` |
| mob effect updates | `18/1/1200/14 → 16/2/300/14 → 8/0/300/14` | `18/1/1200/14 → 16/2/300/14 → 8/0/300/14` |
| mob effect removals | `[]` | `[]` |
| block changed ACK sequences | `[35]` | `[35]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1811` | `3966` |

Effect tuple sırası `effect registry ID / amplifier / duration / flags` biçimindedir;
entity ID her koşuda değişen oyuncu entity'sidir (`1302` ve `1329`). Her iki
console item replacement işlemi başarıyla tamamlandı, slot tüketim sonrası
boşaldı ve server normal disconnect ile kapandı. İlk denemelerde default
pufferfish'in tam açlık nedeniyle tüketilmediği görüldü; çevrimiçi oyuncu
NBT'sindeki `foodLevel` alanı vanilla tarafından doğrudan değiştirilemediği
için bu başarısız koşular kanıt sayılmadı.

## Sonuç ve açık kapsam

İki vanilla oturumu, `FOOD` component ID `23` gövdesinin item stack parser'ı
tarafından tüketildiğini, 32 tick kullanımın ardından stack'in boşaldığını,
ACK'in `35` olarak korunduğunu ve pufferfish effect'lerinin Java'daki üçlü
sıra, süre, amplifier ve flags değerleriyle geldiğini doğrular. Daha fazla
component codec'i, diğer consumable/effect varyantları ve inventory click
açık kalır. Default full-hunger reddi [ayrı birleşik kayıttadır](survival-pufferfish-full-hunger-rejection-conformance.md).

Sözleşme ayrıntıları [survival pufferfish use contract belgesindedir](#sözleşme).
