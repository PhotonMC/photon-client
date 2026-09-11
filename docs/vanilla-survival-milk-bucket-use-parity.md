# P12 — Vanilla 26.2 survival component-bearing milk bucket doğrulaması

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

Sözleşme ayrıntıları [survival milk bucket use contract belgesindedir](survival-milk-bucket-use-contract.md).
