# P12 — Vanilla 26.2 survival food component override'lı pufferfish doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
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
component codec'i, diğer consumable/effect varyantları, default full-hunger
reddi ve inventory click açık kalır.

Sözleşme ayrıntıları [survival pufferfish use contract belgesindedir](survival-pufferfish-use-contract.md).
