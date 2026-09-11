# P12 — Vanilla 26.2 survival enchanted golden apple doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her oyuncu bağlandıktan
sonra slot 0'a vanilla enchanted golden apple verildi; console başarı satırı
görülmeden koşu geçerli sayılmadı.

```text
cargo run --offline --release --example survival_enchanted_golden_apple_use_probe -- 127.0.0.1:25566 RustEnchGolden1
cargo run --offline --release --example survival_enchanted_golden_apple_use_probe -- 127.0.0.1:25566 RustEnchGolden2

item replace entity <player> hotbar.0 with minecraft:enchanted_golden_apple
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustEnchGolden1 | RustEnchGolden2 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1015, components=[]` | `state=2, slot=36, count=1, item=1015, components=[]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container slot after use | `state=3, slot=36, empty` | `state=3, slot=36, empty` |
| mob effect updates | `9/1/400/14 → 10/0/6000/14 → 11/0/6000/14 → 21/3/2400/14` | `9/1/400/14 → 10/0/6000/14 → 11/0/6000/14 → 21/3/2400/14` |
| mob effect removals | `[]` | `[]` |
| block changed ACK sequences | `[37]` | `[37]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1397` | `1706` |

Effect tuple sırası `effect registry ID / amplifier / duration / flags` biçimindedir;
entity ID her koşuda değişen oyuncu entity'sidir (`1444` ve `1478`). Her iki
console item replacement işlemi başarıyla tamamlandı, slot tüketim sonrası
boşaldı ve server normal disconnect ile kapandı. Protocol error/warning
gözlenmedi.

## Sonuç ve açık kapsam

İki vanilla oturumu, patch'siz enchanted golden apple stack'inin 32 tick
sonunda tüketildiğini, ACK'in `37` olarak geldiğini ve dört effect'in Java'daki
sıra, amplifier, süre ve flags değerleriyle uygulandığını doğrular. Diğer
consumable/effect varyantları, effect yenileme ve inventory click açık kalır.

Sözleşme ayrıntıları [survival enchanted golden apple use contract belgesindedir](survival-enchanted-golden-apple-use-contract.md).
