# P13 — Vanilla survival inventory pickup state-id correction doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her koşuda console item
replacement başarı satırı görülmeden sonuç geçerli sayılmadı.

```text
cargo run --offline --release --example survival_inventory_pickup_state_correction_probe -- 127.0.0.1:25566 InvCorr1
cargo run --offline --release --example survival_inventory_pickup_state_correction_probe -- 127.0.0.1:25566 InvCorr2

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken slot 36'ya primary pickup yollar;
state-id'yi kasıtlı olarak `0`, changed-slot map'ini ve carried hash'i boş
tutar. Bu kontrollü eski client tahmini server'ın tıklamayı uyguladıktan sonra
full resync dalına girdiğini ayırt eder.

## Sonuçlar

| Gözlem | InvCorr1 | InvCorr2 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id | `0` | `0` |
| initial container content | `state=1, items=46, slot36=empty, carried=empty` | `state=1, items=46, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot36=empty, carried=count8/item1` | `state=3, items=46, slot36=empty, carried=count8/item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `3064` | `2772` |

Her iki console item replacement işlemi başarıyla tamamlandı. Server slot 36'yı
boşaltıp 8 taşlık carried stack oluşturdu ve eski state-id nedeniyle state `3`
tam content düzeltmesi gönderdi. Vanilla logunda protocol error, action warning
veya timeout gözlenmedi; iki oturumun karşılaştırılan inventory çıktıları
aynıdır.

## Sonuç ve açık kapsam

İki bağımsız vanilla oturumu, `container_click` state-id uyuşmazlığında pickup
işleminin uygulanıp ardından slot ve cursor içeriğinin authoritative full
resync ile düzeltildiğini doğrular. Sözleşme ayrıntıları [survival inventory
pickup state correction contract belgesindedir](survival-inventory-pickup-state-correction-contract.md).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, secondary
pickup, quick-move, swap, throw, drag, double-click, stack sınırı ve özel menu
akışları açık kalır.
