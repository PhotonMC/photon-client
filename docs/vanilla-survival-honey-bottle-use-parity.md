# P12 — Vanilla 26.2 survival component-bearing honey bottle doğrulaması

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

Sözleşme ayrıntıları [survival honey bottle use contract belgesindedir](survival-honey-bottle-use-contract.md).
