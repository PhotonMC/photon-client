# P12 — Survival component-bearing honey bottle use sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun slot 0'daki honey bottle'ı kullanmasını ve 40
tick'lik içme süresinin sonunda glass bottle'a dönüşmesini sınar. Stack ayrıca
`custom_model_data` component patch'i taşır; aynı container wire parser'ı
tüketim öncesi pozitif component'i ve tüketim sonrası boş patch'i gözlemler.

## Java referansı

`Items.java:1930-1933` honey bottle'ı `FOOD` ve `Consumables.HONEY_BOTTLE`,
`usingConvertsTo(GLASS_BOTTLE)` ve maksimum stack 16 ile kaydeder.
`Consumables.java:16-20` tüketim süresini 2.0 saniye (40 tick), animasyonu drink
ve honey drink sesini seçer. Genel `Item.java:189-216` consumable component'i
kullanımı başlatır; `LivingEntity.java:3480-3484,3574-3587` server tick'leri
sonunda consume ve remainder dönüşümünü tamamlar.

`ItemStack.java:122-200` optional stack wire codec'inde count, item registry
ID'si ve component patch listelerini taşır. Bu probe `custom_model_data`
component ID `17` için doğrulanan dört liste codec'ini kullanır; başka pozitif
component türleri bu dar parser kapsamına dahil değildir.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: slot 0'a
  `minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `28`
- Kullanım sonrası drain: 2.5 saniye; bu süre 40 server tick'inin tamamlanmasını
  ve glass bottle dönüşümünün yayınlanmasını bekler

Beklenen invariant'lar şunlardır: carried item packet sayısı `1`, use packet
sayısı `1`, `use_item_on` sayısı `0`, ACK `[28]`; hazırlık slot update'i
count `1`, honey bottle item ID `1412` ve pozitif component `[17]`; bitiş slot
update'i state `3`, slot `36`, count `1`, glass bottle item ID `1149` ve boş
component listeleridir. İlk content update'i state `1` ve 46 slot taşır.

## Sınır

Bu sözleşme honey bottle consumable akışını, component-bearing stack codec'ini
ve glass bottle remainder dönüşümünü kapsar. Poison temizleme efektinin aktif
bir effect üzerindeki sonucu, diğer component codec'leri, diğer consumable'lar,
erken bırakma, iki el önceliği, inventory click ve client prediction sonraki
dilimlerdir.


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
cargo run --offline --release --example survival_honey_bottle_use_probe -- 127.0.0.1:25566 RustHoneyUse1
cargo run --offline --release --example survival_honey_bottle_use_probe -- 127.0.0.1:25566 RustHoneyUse2

item replace entity <player> hotbar.0 with minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustHoneyUse1 | RustHoneyUse2 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1412, components=[17]` | `state=2, slot=36, count=1, item=1412, components=[17]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container slot after use | `state=3, slot=36, count=1, item=1149, components=[]` | `state=3, slot=36, count=1, item=1149, components=[]` |
| block changed ACK sequences | `[28]` | `[28]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2206` | `1859` |

`use_item_on` gönderilmedi; dolu stack doğrudan `use_item` yolundan kullanıldı.
40 tick sonunda server slotu glass bottle'a dönüştürdü ve custom component
patch'i remainder stack'e taşınmadı. Oyuncuda poison effect bulunmadığı için
honey bottle'ın effect temizleme sonucu bu koşuda ayrıca gözlenmedi. Server
logunda iki oturum için item replacement, normal disconnect ve protocol
error/warning olmaması gözlendi.

## Sonuç ve açık kapsam

İki vanilla oturumu, Java'nın honey bottle tüketim başlangıcını, 40 tick
tamamlanmasını, sequence ACK'ini, pozitif component patch'inin wire üzerinden
okunmasını ve glass bottle remainder dönüşümünü aynı invariant'larla verdi.
Aktif poison effect temizliği, `custom_model_data` dışındaki component codec'leri,
diğer consumable/item varyantları, erken bırakma, inventory click ve prediction
açık kalır.

Sözleşme ayrıntıları [survival honey bottle use contract belgesindedir](#sözleşme).
