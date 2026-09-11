# P12 — Vanilla 26.2 survival honey bottle erken bırakma doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her oyuncu oyuna girdikten sonra console üzerinden component-bearing honey bottle stack'i verildi; console başarı satırı görülmeden koşu geçerli sayılmadı.

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

Sözleşme ayrıntıları [survival honey bottle erken bırakma contract belgesindedir](survival-honey-bottle-early-release-contract.md). Tam tüketim [ayrı kayıttadır](vanilla-survival-honey-bottle-use-parity.md); poison temizleme [ayrı kayıttadır](vanilla-survival-honey-bottle-poison-cleanup-parity.md).
