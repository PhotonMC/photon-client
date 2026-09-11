# P13 — Vanilla survival inventory swap state-id correction doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her geçerli koşuda console
item replacement başarı satırı görülmeden sonuç kabul edilmedi.

```text
cargo run --offline --release --example survival_inventory_swap_state_correction_probe -- 127.0.0.1:25566 InvSwap1
cargo run --offline --release --example survival_inventory_swap_state_correction_probe -- 127.0.0.1:25566 InvSwap4

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken menu slotu 9'a button `0` ve
`SWAP` input id `2` ile tıklama yollar; state-id'yi `0`, changed-slot map'ini
ve carried hash'i boş tutar. Böylece server'ın hotbar slotu 36'dan main
inventory slotu 9'a taşıyıp full resync yapması ayırt edilir.

## Sonuçlar

| Gözlem | InvSwap1 | InvSwap4 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / input id | `0 / 2` | `0 / 2` |
| initial container content | `state=1, items=46, slot9=empty, slot36=empty, carried=empty` | `state=1, items=46, slot9=empty, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot9=count8/item1, slot36=empty, carried=empty` | `state=3, items=46, slot9=count8/item1, slot36=empty, carried=empty` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1009` | `1670` |

Her iki geçerli console item replacement işlemi başarıyla tamamlandı. Server
slot 36'daki 8 taş stack'ini menu slotu 9'a taşıdı ve eski state-id nedeniyle
state `3` tam content düzeltmesi gönderdi. Vanilla logunda protocol error,
action warning veya timeout gözlenmedi; iki oturumun karşılaştırılan inventory
çıktıları aynıdır.

İki ara koşul (`InvSwap2`, `InvSwap3`) item replacement komutunun probe'un 12
saniyelik click beklemesinin sonunda işlenmesi nedeniyle geçerli parity koşulu
olarak alınmadı. Bu koşullarda click boş slot üzerinde çalıştı; sonuçlar
authoritative hazırlık gerçekleşmeden yapılan timing yarışını gösterdi.

## Sonuç ve açık kapsam

İki bağımsız geçerli vanilla oturumu, `SWAP` state-id uyuşmazlığında taşımanın
uygulanıp slot 9'daki authoritative stack ile full resync edildiğini doğrular.
Sözleşme ayrıntıları [survival inventory swap state correction contract
belgesindedir](survival-inventory-swap-state-correction-contract.md).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, pickup button
varyantları, throw, drag, double-click, stack sınırı ve özel menu akışları açık
kalır.
