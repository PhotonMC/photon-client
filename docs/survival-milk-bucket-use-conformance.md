# P12 — Survival component-bearing milk bucket use sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun slot 0'daki süt kovasını kullanmasını ve 32 tick'lik
içme süresinin sonunda kovaya dönüşmesini sınar. Stack ayrıca `custom_model_data`
component patch'i taşır; amaç item stack wire codec'inde pozitif component'in
tüketim öncesi okunması ve dönüşen remainder stack'inin patch'siz gözlenmesidir.

## Java referansı

`Items.java:1254-1257` süt kovasını tekli stack, `Consumables.MILK_BUCKET` ve
`usingConvertsTo(BUCKET)` ile kaydeder. `Consumables.java:65,72-74` süt kovasının
1.6 saniyelik (32 tick) drink consumable olduğunu ve consume efektlerinin
durum efektlerini temizlediğini belirler. Genel item yolu `Item.java:189-216`
component'i bulup kullanımı başlatır ve tamamlandığında component'in
`onConsume` yolunu çalıştırır; `LivingEntity.java:3480-3484,3574-3587` server
tick'inde sayacı azaltıp tamamlanan stack'i eldeki item'a yazar.

`ItemStack.java:122-200` optional stack wire codec'inde count, item registry
ID'si ve component patch count/listelerini taşır. `DataComponents.java:154-156`
ile kayıtlı `custom_model_data` component'i bu koşuda pozitif component ID `17`
olarak gelir. Rust probe bu component'in dört listesini tüketir ve pozitif/
negatif component ID'lerini container gözlemine yazar; diğer pozitif component
codec'leri henüz bu dar dilimin dışındadır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: slot 0'a
  `minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `27`
- Kullanım sonrası drain: 2.5 saniye; bu süre 32 server tick'inin tamamlanmasını
  ve inventory dönüşümünün yayınlanmasını bekler

Beklenen invariant'lar şunlardır: carried item packet sayısı `1`, use packet
sayısı `1`, `use_item_on` sayısı `0`, ACK `[27]`; hazırlık slot update'i
count `1`, milk bucket item ID `1046` ve pozitif component `[17]`; bitiş slot
update'i state `3`, slot `36`, count `1`, bucket item ID `1040` ve boş component
listeleridir. İlk content update'i state `1` ve 46 slot taşır.

## Sınır

Bu sözleşme milk bucket'ın yerleşik consumable akışını, component-bearing stack
codec'ini ve bucket remainder dönüşümünü kapsar. Component değerlerini oyun
renderer'ında kullanmak, diğer component türlerini decode etmek, status effect
uygulama/temizleme sonucu, farklı consumable'lar, erken bırakma, iki el önceliği,
inventory click ve client prediction sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her oyuncu oyuna girdikten
sonra console üzerinden aşağıdaki stack verildi; console başarı satırı
görülmeden koşu geçerli sayılmadı.

```text
cargo run --offline --release --example survival_milk_bucket_use_probe -- 127.0.0.1:25566 RustMilkUse1
cargo run --offline --release --example survival_milk_bucket_use_probe -- 127.0.0.1:25566 RustMilkUse2

item replace entity <player> hotbar.0 with minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustMilkUse1 | RustMilkUse2 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1046, components=[17]` | `state=2, slot=36, count=1, item=1046, components=[17]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container slot after use | `state=3, slot=36, count=1, item=1040, components=[]` | `state=3, slot=36, count=1, item=1040, components=[]` |
| block changed ACK sequences | `[27]` | `[27]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `3267` | `3522` |

`use_item_on` gönderilmedi; dolu stack doğrudan `use_item` yolundan kullanıldı.
İçme süresi sonunda server `finishUsingItem` sonucu slotu bucket'a çevirdi ve
custom component patch'i remainder stack'e taşınmadı. Peaceful başlangıç ve
durum efekti olmayan oyuncu nedeniyle bu koşu effect temizleme içeriğini ayrıca
ayırt etmez. Server logunda iki oturum için item replacement, normal disconnect
ve protocol error/warning olmaması gözlendi.

## Sonuç ve açık kapsam

İki vanilla oturumu, Java'nın component-backed milk bucket kullanım başlangıcını,
32 tick tamamlanmasını, sequence ACK'ini, pozitif component patch'inin wire
üzerinden okunmasını ve `BUCKET` remainder dönüşümünü aynı invariant'larla verdi.
`custom_model_data` dışındaki component codec'leri, diğer consumable/item
varyantları, erken bırakma, effect sonucu, inventory click ve prediction açık
kalır.

Sözleşme ayrıntıları [survival milk bucket use contract belgesindedir](#sözleşme).
