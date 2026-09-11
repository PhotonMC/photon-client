# P12 — Vanilla 26.2 survival pufferfish full-hunger rejection doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Yeni oyuncuların food
seviyesi `20` iken slot 0'a patch'siz pufferfish verildi; console başarı satırı
görülmeden koşu geçerli sayılmadı.

```text
cargo run --offline --release --example survival_pufferfish_full_hunger_rejection_probe -- 127.0.0.1:25566 RustPufferFull1
cargo run --offline --release --example survival_pufferfish_full_hunger_rejection_probe -- 127.0.0.1:25566 RustPufferFull2

item replace entity <player> hotbar.0 with minecraft:pufferfish
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustPufferFull1 | RustPufferFull2 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1089, components=[]` | `state=2, slot=36, count=1, item=1089, components=[]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container slot updates after use | `[]` | `[]` |
| mob effect updates | `[]` | `[]` |
| mob effect removals | `[]` | `[]` |
| block changed ACK sequences | `[38]` | `[38]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2293` | `2709` |

Her iki console item replacement işlemi başarıyla tamamlandı. ACK `38` geldi;
slotta tüketim veya component değişimi yayınlanmadı, effect listeleri boş kaldı
ve server normal disconnect ile kapandı. Protocol error/warning gözlenmedi.

## Sonuç ve açık kapsam

İki vanilla oturumu, default pufferfish'in tam food seviyesinde Java tarafından
reddedildiğini; sequence ACK'inin yine gönderildiğini, ancak consumable başlangıcı,
stack tüketimi ve üç status effect yayınının oluşmadığını doğrular. Düşük food
seviyesinde kullanım ve food component override'ı [ayrı parity kaydında](vanilla-survival-pufferfish-use-parity.md),
diğer consumable/effect varyantları ve inventory click açık kalır.

Sözleşme ayrıntıları [survival pufferfish full-hunger rejection contract belgesindedir](survival-pufferfish-full-hunger-rejection-contract.md).
