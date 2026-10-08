# P12 — Survival pufferfish full-hunger rejection sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, default pufferfish'in survival oyuncu tam açlıktayken
kullanılmamasını sınar. `use_item` sequence ACK'i alınır; consumable kullanım
durumu başlamaz, stack tüketilmez ve pufferfish effect'leri uygulanmaz.

## Java referansı

`Items.java:1321-1325` pufferfish'i `Foods.PUFFERFISH` ile kaydeder.
`Foods.java:33` food'un `can_always_eat` değerini false bırakır. Player'ın
`canEat` yolu `Player.java:1600-1606` içinde yalnızca invulnerable, always-eat
veya `FoodData.needsFood()` koşullarından biri true olduğunda başarılıdır;
`FoodData.java:88-98` tam food seviyesinin `20` olduğunu tanımlar.

Server listener `ServerGamePacketListenerImpl.java:1417-1443` sequence'i item
davranışından önce ACK kuyruğuna alır. `Item.java:180-194` consumable başlangıcı
ancak item kullanımı kabul edildiğinde çalışır; bu senaryoda `canEat` false
olduğu için tüketim tamamlanma yolu açılmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`, yeni oyuncunun başlangıç food seviyesi `20`
- Sunucu console hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:pufferfish`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `38`
- Kullanım sonrası drain: 2.5 saniye; reddin ve başlangıç paketlerinin gözlenmesi beklenir

Beklenen invariant'lar carried/use packet sayılarının `1/1` olması, hazırlık
slotunun state `2`, slot `36`, count `1`, item ID `1089`, component `[]`
taşıması, başlangıç content'inin state `1` ve 46 slot olması, ACK listesinin
`[38]` olmasıdır. Kullanım sonrasında ek slot update'i, effect update'i ve
effect remove'u beklenmez; default stack tüketilmeden kalır.

## Sınır

Bu sözleşme yalnızca full-hunger eligibility reddini kapsar. Food seviyesi
düşük pufferfish tüketimi [food component override sözleşmesinde](survival-pufferfish-use-conformance.md),
diğer food/effect varyantları ve inventory click sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
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
seviyesinde kullanım ve food component override'ı [ayrı birleşik kayıtta](survival-pufferfish-use-conformance.md),
diğer consumable/effect varyantları ve inventory click açık kalır.

Sözleşme ayrıntıları [survival pufferfish full-hunger rejection contract belgesindedir](#sözleşme).
