# P13 — Vanilla survival inventory quick-move state-id correction doğrulaması

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
cargo run --offline --release --example survival_inventory_quick_move_state_correction_probe -- 127.0.0.1:25566 InvQuick1
cargo run --offline --release --example survival_inventory_quick_move_state_correction_probe -- 127.0.0.1:25566 InvQuick2

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken slot 36'ya button `0` ve
`QUICK_MOVE` input id `1` ile tıklama yollar; state-id'yi `0`, changed-slot
map'ini ve carried hash'i boş tutar. Böylece server'ın stack'i ilk boş main
inventory slotu olan menu slotu 9'a taşıyıp full resync yapması ayırt edilir.

## Sonuçlar

| Gözlem | InvQuick1 | InvQuick2 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / input id | `0 / 1` | `0 / 1` |
| initial container content | `state=1, items=46, slot9=empty, slot36=empty, carried=empty` | `state=1, items=46, slot9=empty, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot9=count8/item1, slot36=empty, carried=empty` | `state=3, items=46, slot9=count8/item1, slot36=empty, carried=empty` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1216` | `1512` |

Her iki console item replacement işlemi başarıyla tamamlandı. Server slot 36'daki
8 taş stack'ini slot 9'a taşıdı ve eski state-id nedeniyle state `3` tam content
düzeltmesi gönderdi. Vanilla logunda protocol error, action warning veya
timeout gözlenmedi; iki oturumun karşılaştırılan inventory çıktıları aynıdır.

## Sonuç ve açık kapsam

İki bağımsız vanilla oturumu, `QUICK_MOVE` state-id uyuşmazlığında shift-click
taşımasının uygulanıp slot 9'daki authoritative stack ile full resync edildiğini
doğrular. Sözleşme ayrıntıları [survival inventory quick-move state correction
contract belgesindedir](survival-inventory-quick-move-state-correction-contract.md).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, primary ve
secondary pickup, swap, throw, drag, double-click, stack sınırı ve özel menu
akışları açık kalır.
