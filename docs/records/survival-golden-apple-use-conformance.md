# P12 — Survival golden apple use sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun golden apple'ı tüketmesini, 32 tick'lik food
kullanımını, stack tüketimini ve golden apple'a bağlı iki status effect'in wire
sırasını sınar. Golden apple vanilla food tanımında her zaman yenebilir ve
item stack'i pozitif component patch'i taşımaz.

## Java referansı

`Items.java:1124-1129` golden apple'ı `Foods.GOLDEN_APPLE` ve
`Consumables.GOLDEN_APPLE` ile kaydeder. `Foods.java:24` food'un her zaman
yenebilir olduğunu, `Consumables.java:38-44` ise tüketim sonunda regeneration
ve absorption effect'lerini bu sırayla eklediğini gösterir. Default food
consumable süresi `Consumables.java:68-74` içinde 1.6 saniye, yani 32 tick'tir.

`LivingEntity.java:3480-3484,3574-3587` kullanım süresi bitince stack'i
tamamlar. `ApplyStatusEffectsConsumeEffect.java:15-63` effect listesini
sıralı olarak uygular. ItemStack wire codec'i `ItemStack.java:122-200`
count, item registry ID'si ve component patch listesini taşır; vanilla golden
apple hazırlık stack'inde patch listesi boştur.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:golden_apple`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `36`
- Kullanım sonrası drain: 2.5 saniye; bu süre 32 server tick'inin tamamlanmasını
  ve effect/slot yayınlarını bekler

Beklenen invariant'lar carried/use packet sayılarının `1/1` olması, hazırlık
slotunun state `2`, slot `36`, count `1`, item ID `1014`, component `[]`
taşıması, başlangıç content'inin state `1` ve 46 slot olması, bitiş slotunun
boş yayınlanması ve ACK listesinin `[36]` olmasıdır. Effect update'leri aynı
oyuncu entity ID'siyle şu sırayı korur: regeneration `9`, amplifier `1`,
duration `100`, flags `14`; absorption `21`, amplifier `0`, duration `2400`,
flags `14`. Bu akışta remove effect paketi beklenmez.

## Sınır

Bu sözleşme golden apple consumable akışını, patch'siz item stack'i ve iki
effect update sırasını kapsar. Enchanted golden apple, diğer consumable/effect
varyantları, effect yenileme, iki el önceliği ve inventory click sonraki
dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her oyuncu bağlandıktan
sonra slot 0'a vanilla golden apple verildi; console başarı satırı görülmeden
koşu geçerli sayılmadı.

```text
cargo run --offline --release --example survival_golden_apple_use_probe -- 127.0.0.1:25566 RustGoldenH1
cargo run --offline --release --example survival_golden_apple_use_probe -- 127.0.0.1:25566 RustGoldenH2

item replace entity <player> hotbar.0 with minecraft:golden_apple
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustGoldenH1 | RustGoldenH2 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1014, components=[]` | `state=2, slot=36, count=1, item=1014, components=[]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container slot after use | `state=3, slot=36, empty` | `state=3, slot=36, empty` |
| mob effect updates | `9/1/100/14 → 21/0/2400/14` | `9/1/100/14 → 21/0/2400/14` |
| mob effect removals | `[]` | `[]` |
| block changed ACK sequences | `[36]` | `[36]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `3027` | `3465` |

Effect tuple sırası `effect registry ID / amplifier / duration / flags` biçimindedir;
entity ID her koşuda değişen oyuncu entity'sidir (`1371` ve `1404`). Her iki
console item replacement işlemi başarıyla tamamlandı, slot tüketim sonrası
boşaldı ve server normal disconnect ile kapandı. Protocol error/warning
gözlenmedi.

## Sonuç ve açık kapsam

İki vanilla oturumu, patch'siz golden apple stack'inin 32 tick sonunda
tüketildiğini, ACK'in `36` olarak geldiğini ve regeneration ile absorption
effect'lerinin Java'daki sıra, amplifier, süre ve flags değerleriyle uygulandığını
doğrular. Enchanted golden apple, diğer consumable/effect varyantları, effect
yenileme ve inventory click açık kalır.

Sözleşme ayrıntıları [survival golden apple use contract belgesindedir](#sözleşme).
