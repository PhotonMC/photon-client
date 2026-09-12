# P13 — [server-conformance] Item component codec conformance

Bu kayıt, daha önce ayrı dosyalarda tutulan item component doğru prediction
koşullarını tek parametrik probe ve tek tablo altında birleştirir. Kanıt sınıfı
`server-conformance`'dır: Rust'ın gönderdiği `HashedStack` ve component patch'inin
Mojang vanilla sunucusu tarafından kabul edildiğini gösterir. Java istemcisi ile
Rust istemcisinin aynı girdide eşitliğini göstermez ve P13'ü kapatmaz.

## Sözleşme

Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Sunucu, Mojang'ın
26.2 dedicated vanilla sunucusudur; sabit SHA-1 değeri
`823e2250d24b3ddac457a60c92a6a941943fcd6a`'dır. Her koşulda survival player
inventory menu (`container_id=0`) kullanılır.

Hazırlık komutu listedeki item stack'i hotbar index `0` / menu slotu `36` içine
koyar. Probe, başlangıç content paketinden sonra bu itemi ve listedeki positive
component type id'lerini gözler; hazırlık paketindeki gerçek state-id'yi click'e
taşır. Click `slot=36`, `button=0`, `PICKUP input=0` değerlerini kullanır.

Changed-slot prediction, pickup sonrasında boş slotu taşır. Carried prediction,
count `1`, item registry id'si ve tabloda belirtilen component hash'lerini taşır;
`max_damage` satırında Java'nın gönderdiği iki component (`max_stack_size` ve
`max_damage`) birlikte bulunur. Başarı ölçütü, vanilla sunucunun click'i
uygulaması, beklenen cursor/slot düzeltmesini üretmemesi ve hazırlık dışı full
content resync göndermemesidir.

| Component path | Type id | Codec | Item id | Carried hash | Hazırlık stack'i |
| --- | ---: | --- | ---: | --- | --- |
| `custom_model_data` | 17 | `CustomModelData` | 1046 | `0x77a4a24f` | `minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]` |
| `food` | 23 | `Food` | 1089 | `0xfac69b4d` | `minecraft:pufferfish[food={nutrition:1,saturation:0.1,can_always_eat:true}]` |
| `damage` | 3 | `VarInt` | 1 | `0x5d50d71c` | `minecraft:stone[damage=1]` |
| `tool` | 28 | `Tool` | 1 | `0x37de0062` | `minecraft:stone[tool={rules:[]}]` |
| `weapon` | 29 | `Weapon` | 1 | `0xc574b4c8` | `minecraft:stone[weapon={item_damage_per_attack:1,disable_blocking_for_seconds:0.0}]` |
| `max_stack_size` | 1 | `VarInt` | 1 | `0x5d50d71c` | `minecraft:stone[max_stack_size=1]` |
| `max_damage` | 2 | `VarInt` | 1 | `0x5d50d71c` | `minecraft:stone[max_stack_size=1,max_damage=1]` |
| `unbreakable` | 4 | `Unit` | 1 | `0xc574b4c8` | `minecraft:stone[unbreakable={}]` |
| `minimum_attack_charge` | 7 | `Float` | 1 | `0x9ec7d769` | `minecraft:stone[minimum_attack_charge=0.5]` |
| `repair_cost` | 19 | `VarInt` | 1 | `0x5d50d71c` | `minecraft:stone[repair_cost=1]` |
| `enchantment_glint_override` | 21 | `Byte` | 1 | `0xc336cec2` | `minecraft:stone[enchantment_glint_override=true]` |
| `rarity` | 12 | `VarInt` | 1 | `0xab53df3a` | `minecraft:stone[rarity=rare]` |
| `item_model` | 10 | `Identifier` | 1 | `0x75c93192` | `minecraft:stone[item_model="minecraft:dirt"]` |
| `use_effects` | 5 | `UseEffects` | 1 | `0x9619e224` | `minecraft:stone[use_effects={can_sprint:true,interact_vibrations:false,speed_multiplier:1.0}]` |
| `damage_type` | 8 | `RegistryId` | 1 | `0x55e33a50` | `minecraft:stone[damage_type=generic]` |
| `glider` | 34 | `Unit` | 1 | `0xc574b4c8` | `minecraft:stone[glider={}]` |
| `tooltip_style` | 35 | `Identifier` | 1 | `0xababf3db` | `minecraft:stone[tooltip_style="minecraft:default"]` |
| `enchantable` | 31 | `VarInt` | 1 | `0xdeed7aaa` | `minecraft:stone[enchantable={value:10}]` |
| `dye` | 43 | `VarInt` | 1 | `0x8c6452aa` | `minecraft:stone[dye=red]` |
| `dyed_color` | 44 | `Int` | 1 | `0x23f65927` | `minecraft:stone[dyed_color=16711680]` |
| `potion_duration_scale` | 52 | `Float` | 1 | `0x9a13eb94` | `minecraft:stone[potion_duration_scale=1.5]` |
| `map_color` | 45 | `Int` | 1 | `0x23f65927` | `minecraft:stone[map_color=16711680]` |
| `map_id` | 46 | `VarInt` | 1 | `0x5d50d71c` | `minecraft:stone[map_id=1]` |
| `base_color` | 73 | `VarInt` | 1 | `0x8c6452aa` | `minecraft:stone[base_color=red]` |
| `note_block_sound` | 71 | `Identifier` | 1 | `0xda545935` | `minecraft:stone[note_block_sound="minecraft:block.note_block.harp"]` |
| `ominous_bottle_amplifier` | 63 | `VarInt` | 1 | `0x3f725e25` | `minecraft:stone[ominous_bottle_amplifier=2]` |
| `break_sound` | 81 | `RegistryId` | 1 | `0x9662f221` | `minecraft:stone[break_sound="minecraft:block.stone.break"]` |
| `wolf/collar` | 85 | `VarInt` | 1 | `0x8c6452aa` | `minecraft:stone[wolf/collar=red]` |
| `tropical_fish/base_color` | 90 | `VarInt` | 1 | `0x8c6452aa` | `minecraft:stone[tropical_fish/base_color=red]` |
| `tropical_fish/pattern` | 89 | `VarInt` | 1 | `0xe196874b` | `minecraft:stone[tropical_fish/pattern=stripey]` |
| `tropical_fish/pattern_color` | 91 | `VarInt` | 1 | `0x8c6452aa` | `minecraft:stone[tropical_fish/pattern_color=red]` |
| `cat/collar` | 108 | `VarInt` | 1 | `0x8c6452aa` | `minecraft:stone[cat/collar=red]` |
| `sheep/color` | 109 | `VarInt` | 1 | `0x8c6452aa` | `minecraft:stone[sheep/color=red]` |

`DyeColor` tabanlı satırlar color id `14` kullanır. Tropical fish pattern satırı
`STRIPEY` için packed id `257` kullanır. `damage_type` satırında
`minecraft:generic` registry id `27`'dir. `break_sound` satırındaki hash,
doğrudan holder hash'i değil, sound registry id `1596` ile registry-aware Java
oracle sonucudur.

## Java incelemesi

Java kaynak incelemesi `DataComponents` kayıt sırasını ve component codec'lerini;
`DyeColor`, `TropicalFish`, `Cat` ve `Sheep` ise değerlerin persistent ve network
biçimlerini doğruladı. Hash değerleri Rust çıktısından türetilmedi; aynı sabit
Java 26.2 / Java 25 oracle girdileriyle üretildi.

`wolf/collar`, tropical fish alt yolları, `cat/collar` ve `sheep/color` aynı
`DyeColor.STREAM_CODEC` dalını kullandığı için ayrı Rust enum dalı açılmadı.
`shulker/color` da aynı codec'i kullanır ve yeni bir gözlenmiş dal üretmediği için
tabloya yeni probe olarak eklenmedi. Yeni bir satır ancak Java kaynaklarında yeni
codec/branch veya önceki kayıtlardan farklı bir gözlem ortaya çıkarsa eklenir.

## Vanilla kanıtı

Önceki ayrı doğrulama dosyalarındaki geçerli iki koşu bu kayda taşındı. Her satır
iki bağımsız vanilla oturumunda `container_click_packets_sent=1` ve
`chunk packet / decoded / batch finished=157 / 157 / 51` verdi; geçerli koşul
sonuçlarında click sonrası incremental slot/cursor update sayısı `0`, ek full
content gözlemi `0` ve server logunda protocol error yoktu. Hazırlık komutu
uygulanmadan çalışan `InvNoPrep2C26` koşusu başarı kabul edilmedi; probe artık
30 saniyelik hazırlık penceresi sonunda `item component preparation` codec
hatası döndürür. Guard sonrasında gerçek hazırlıkla çalışan `InvPrep2C26`
koşusu `damage` satırını yeniden doğruladı ve aynı `1 / 157 / 157 / 51`
ölçümlerini verdi.

| Component path | Geçerli kayıtlar |
| --- | --- |
| `custom_model_data` | `InvCompOk3`, `InvCompOk4` |
| `food` | `InvFood2`, `InvFood3` |
| `damage` | `InvDamage2`, `InvDamage3`, `CompDamageC26`, `InvPrep2C26` |
| `tool` | `InvTool2`, `InvTool3` |
| `weapon` | `InvWeapon2`, `InvWeapon3` |
| `max_stack_size` | `InvMaxStack2`, `InvMaxStack4` |
| `max_damage` | `InvMaxDamage2`, `InvMaxDamage3` |
| `unbreakable` | `InvUnbreakable2`, `InvUnbreakable3` |
| `minimum_attack_charge` | `InvMinChargeA`, `InvMinChargeB` |
| `repair_cost` | `InvRepairCostA`, `InvRepairCostB` |
| `enchantment_glint_override` | `InvGlintC`, `InvGlintD` |
| `rarity` | `InvRarityB`, `InvRarityC` |
| `item_model` | `InvItemModelB`, `InvItemModelC` |
| `use_effects` | `InvUseEffectsB`, `InvUseEffectsC` |
| `damage_type` | `InvDamageTypeC`, `InvDamageTypeD` |
| `glider` | `InvGliderB`, `InvGliderC` |
| `tooltip_style` | `InvTooltipStyleB`, `InvTooltipStyleC` |
| `enchantable` | `InvEnchantableB`, `InvEnchantableC` |
| `dye` | `InvDyeB`, `InvDyeC` |
| `dyed_color` | `DyedColorB26`, `DyedColorC26` |
| `potion_duration_scale` | `PotionScaleB26`, `PotionScaleC26` |
| `map_color` | `MapColorB26`, `MapColorC26` |
| `map_id` | `MapIdB26`, `MapIdC26` |
| `base_color` | `BaseColorE26`, `BaseColorF26` |
| `note_block_sound` | `NoteSoundB26`, `NoteSoundC26` |
| `ominous_bottle_amplifier` | `OminousAmpB26`, `OminousAmpC26` |
| `break_sound` | `BreakSoundC26`, `BreakSoundD26` |
| `wolf/collar` | `WolfCollarC26`, `WolfCollarD26` |
| `tropical_fish/base_color` | `TropicalBaseC26`, `TropicalBaseD26` |
| `tropical_fish/pattern` | `TropPatternC26`, `TropPatternD26` |
| `tropical_fish/pattern_color` | `TropPCrF26`, `TropPCrG26` |
| `cat/collar` | `CatCollarC26`, `CatCollarD26` |
| `sheep/color` | `SheepColorC26`, `SheepColorD26` |

Bu koşuların ortak komut biçimi şöyledir; tekil değerler yukarıdaki tabloda
verilmiştir:

```text
cargo run --offline --release --example survival_inventory_item_component_conformance_probe -- 127.0.0.1:25566 <username> <component_path>
```

`cat/collar`, `sheep/color` ve tropical fish satırlarında hazırlık state-id'si
önceki entity component gözlemlerinden etkilenebildiği için probe sabit `2`
kullanmaz; eşleşen son slot gözlemindeki state-id'yi kullanır. Bu, aynı codec'in
state sıralaması gözlemidir ve yeni component varyantı değildir.

## Açık kapsam

Bu kayıt Rust parser'ının tabloda bulunan codec gövdelerini tüketmesini ve
server'a gönderilen hashed component patch'lerini kapsar. İstemci tarafı
container menu/slot modeli, Java istemcisi ile `client-parity`, özel menüler,
değişen slot haritaları ve gerçek oynanabilir istemci akışı hâlâ açıktır.

`SurvivalInventoryAheadComponentCorrectPrediction` ayrı tutulur; onun farkı
component değerinden değil, bilerek ileri container state-id dalından gelir.
