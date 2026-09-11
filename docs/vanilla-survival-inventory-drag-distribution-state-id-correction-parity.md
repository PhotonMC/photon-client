# P13 — Vanilla survival inventory drag distribution state-id correction doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Her oturumda player bağlandıktan sonra vanilla server console'undan şu hazırlık
komutu gönderildi:

```text
item replace entity <player> hotbar.0 with minecraft:stone 8
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_drag_distribution_state_correction_probe -- 127.0.0.1:25566 InvDragDist2
cargo run --offline --release --example survival_inventory_drag_distribution_state_correction_probe -- 127.0.0.1:25566 InvDragDist3
```

İlk `InvDragDist1` denemesi, probe'un 30 saniyelik beklemesi nedeniyle vanilla
keepalive zaman aşımına uğradı ve parity sonucu olarak kullanılmadı. Bekleme 20
saniyeye indirildikten sonraki iki koşu aşağıdaki temiz sonuçları verdi.

## Sonuçlar

| Gözlem | InvDragDist2 | InvDragDist3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `5` | `5` |
| gönderilen state-id | `0` | `0` |
| gönderilen slot/button/input dizisi | `36/0/0`, `-999/0/5`, `9/1/5`, `36/1/5`, `-999/2/5` | `36/0/0`, `-999/0/5`, `9/1/5`, `36/1/5`, `-999/2/5` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| ara full resync'ler | `state=3..6, carried=count8,item1` | `state=3..6, carried=count8,item1` |
| son authoritative full resync | `state=7, slot9=count4,item1, slot36=count4,item1, carried=empty` | `state=7, slot9=count4,item1, slot36=count4,item1, carried=empty` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2212` | `3137` |

Her iki vanilla oturumunda beş stale state-id `0` click'i uygulandı; ilk click
stack'i cursor'a aldı, üç quick-craft paketi carried stack'i korudu ve son
paket count `8` stack'ini iki seçili slota floor bölümüyle `4 + 4` dağıttı.
Vanilla son full content state `7` ile iki slotu ve boş cursor'ı birlikte
bildirdi. Server logunda bu koşular için protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, survival type `0` iki-slot drag
dağıtımının packet sırasını, stale state-id sonrası beş full resync artışını ve
son slot/cursor içeriğini doğrular. Sözleşme ayrıntıları [survival inventory
drag distribution state-id correction contract belgesindedir](survival-inventory-drag-distribution-state-correction-contract.md).

Doğru state-id ile client'ın hashed prediction'ı, diğer quick-craft
type/header'ları, yanlış state'in diğer varyantları, double-click, stack sınırı
ve özel menu akışları açık kalır.
