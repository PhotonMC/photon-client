# P12 — Vanilla 26.2 survival active multi-effect milk bucket doğrulaması

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

Sözleşme ayrıntıları [survival active multi-effect milk bucket cleanup contract belgesindedir](survival-milk-bucket-clear-effects-contract.md).
