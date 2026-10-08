# P12 — Survival honey bottle erken bırakma sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun component-bearing honey bottle kullanımını başlatıp 40 tick'lik tüketim süresi dolmadan bırakmasını sınar. Amaç `RELEASE_USE_ITEM` action'ının kullanım durumunu kapatmasını, honey bottle'ı tüketmemesini ve yalnızca kullanım başlangıcındaki stack'in korunmasını gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

`Items.java:1930-1933` honey bottle'ı 2.0 saniyelik `Consumables.HONEY_BOTTLE` ile kaydeder. `Consumables.java:16-20` bu süreyi 40 tick yapar. `MultiPlayerGameMode.java:514-518` bırakmada `RELEASE_USE_ITEM`, `BlockPos.ZERO`, `Direction.DOWN` ve varsayılan sequence `0` gönderir; ardından yerel kullanım durumunu bırakır.

Server listener `RELEASE_USE_ITEM` action'ında `LivingEntity.releaseUsingItem` çağırıp action handler'ından ACK yoluna girmeden döner. `LivingEntity.java:3611-3622`, item'in `releaseUsing` davranışını çalıştırıp kullanım state'ini kapatır; honey bottle'ın temel `Item.releaseUsing` yolu başarı döndürmediği için `completeUsingItem` çağrılmaz ve stack tüketilmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: slot 0'a `minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `34`
- Erken bırakma: 250 ms sonra `RELEASE_USE_ITEM`, action ordinal `5`, `BlockPos.ZERO`, `DOWN`, sequence `0`
- Tüketim süresi: 40 tick; bırakma bu süreden önce yapılır

Beklenen invariant'lar carried item packet sayısı `1`, use packet sayısı `1`, player action packet sayısı `1`, ACK `[34]`; hazırlık slot update'i count `1`, honey bottle item ID `1412` ve pozitif component `[17]`; kullanım sonrasında slot/content dönüşümü bulunmamasıdır. `RELEASE_USE_ITEM` için ayrı block-change ACK'i beklenmez. İlk content update'i state `1` ve 46 slot taşır.

## Sınır

Bu sözleşme honey bottle'ın erken bırakılmasını ve component-bearing stack'in korunmasını kapsar. Tam tüketim, aktif poison temizleme, başka `useOnRelease` item'leri, iki el önceliği, kullanım sırasında saldırı ve inventory click sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her oyuncu oyuna girdikten sonra console üzerinden component-bearing honey bottle stack'i verildi; console başarı satırı görülmeden koşu geçerli sayılmadı.

```text
cargo run --offline --release --example survival_honey_bottle_early_release_probe -- 127.0.0.1:25566 RustHRelease1
cargo run --offline --release --example survival_honey_bottle_early_release_probe -- 127.0.0.1:25566 RustHRelease2

item replace entity <player> hotbar.0 with minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustHRelease1 | RustHRelease2 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| player action packets sent | `1` | `1` |
| container preparation | `state=2, slot=36, count=1, item=1412, components=[17]` | `state=2, slot=36, count=1, item=1412, components=[17]` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container updates after release | `[]` | `[]` |
| block changed ACK sequences | `[34]` | `[34]` |
| chunk packet / decoded / batch finished | `54 / 54 / 16` | `54 / 54 / 16` |
| received packet | `658` | `1071` |

`use_item` sonrasında 250 ms beklenip `RELEASE_USE_ITEM` gönderildi. Server logunda her iki console item replacement işlemi, normal disconnect ve protocol error/warning olmaması gözlendi. Erken bırakmadan sonra slotta glass bottle veya count azalması yayınlanmadı; `RELEASE_USE_ITEM` için ayrıca block-change ACK'i gelmedi.

## Sonuç ve açık kapsam

İki vanilla oturumu, 40 tick tamamlanmadan bırakılan component-bearing honey bottle'ın tüketilmediğini, kullanım ACK'inin korunup release action'ının ayrı ACK üretmediğini ve slot stack'inin aynı kaldığını doğrular. Tam tüketim, aktif poison temizleme, başka `useOnRelease` item'leri, iki el önceliği, kullanım sırasında saldırı ve inventory click açık kalır.

Sözleşme ayrıntıları [survival honey bottle erken bırakma contract belgesindedir](#sözleşme). Tam tüketim [ayrı kayıttadır](survival-honey-bottle-use-conformance.md); poison temizleme [ayrı kayıttadır](survival-honey-bottle-poison-cleanup-conformance.md).
