# P12 — Survival active multi-effect milk bucket cleanup sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Oyuncu bağlandıktan sonra
üç effect ve component-bearing milk bucket console üzerinden verildi; her
console başarı satırı görülmeden koşu geçerli sayılmadı.

```text
cargo run --offline --release --example survival_milk_bucket_use_probe -- 127.0.0.1:25566 RustMilkClear2
cargo run --offline --release --example survival_milk_bucket_use_probe -- 127.0.0.1:25566 RustMilkClear3

effect give <player> minecraft:poison 60 0 true
effect give <player> minecraft:hunger 60 1 true
effect give <player> minecraft:regeneration 60 0 true
item replace entity <player> hotbar.0 with minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustMilkClear2 | RustMilkClear3 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1046, components=[17]` | `state=2, slot=36, count=1, item=1046, components=[17]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container slot after use | `state=3, slot=36, count=1, item=1040, components=[]` | `state=3, slot=36, count=1, item=1040, components=[]` |
| mob effect updates | `18/0/1200/8 → 16/1/1200/8 → 9/0/1200/8` | `18/0/1200/8 → 16/1/1200/8 → 9/0/1200/8` |
| mob effect removals | `16 → 18 → 9` | `16 → 18 → 9` |
| block changed ACK sequences | `[27]` | `[27]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2483` | `3050` |

Remove effect tuples yalnızca registry ID sırasını gösterir; entity ID iki
koşuda oyuncuya göre değişir (`1605` ve `1648`). Her iki item replacement
başarılı oldu, bucket remainder yayınlandı, üç remove packet'i aynı sırayla
geldi ve server normal disconnect ile kapandı. Protocol error/warning gözlenmedi.

## Sonuç ve açık kapsam

İki vanilla oturumu, milk bucket'ın 32 tick sonunda bucket'a dönüştüğünü,
başlangıçta aktif üç effect'in tümünün kaldırıldığını ve remove packet sırasının
Java effect map traversal sonucu `16 → 18 → 9` olduğunu doğrular. Başka effect
kombinasyonları, diğer component/consumable varyantları, erken bırakma ve
inventory click açık kalır.

Sözleşme ayrıntıları [survival active multi-effect milk bucket cleanup contract belgesindedir](#sözleşme).
