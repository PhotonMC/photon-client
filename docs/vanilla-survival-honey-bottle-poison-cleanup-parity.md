# P12 — Vanilla 26.2 survival honey bottle poison temizleme doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
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

Sözleşme ayrıntıları [survival honey bottle poison cleanup contract belgesindedir](survival-honey-bottle-poison-cleanup-contract.md).
