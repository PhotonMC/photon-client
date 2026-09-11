# P13 — Vanilla survival inventory secondary pickup state-id correction doğrulaması

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
cargo run --offline --release --example survival_inventory_secondary_pickup_state_correction_probe -- 127.0.0.1:25566 InvHalf1
cargo run --offline --release --example survival_inventory_secondary_pickup_state_correction_probe -- 127.0.0.1:25566 InvHalf4

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken slot 36'ya button `1` ve `PICKUP`
input id `0` ile tıklama yollar; state-id'yi `0`, changed-slot map'ini ve
carried hash'i boş tutar. Böylece server'ın 8 taşlık stack'in 4 item'ini
cursor'a alıp slotta 4 bırakması ve full resync yapması ayırt edilir.

## Sonuçlar

| Gözlem | InvHalf1 | InvHalf4 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / button / input id | `0 / 1 / 0` | `0 / 1 / 0` |
| initial container content | `state=1, items=46, slot36=empty, carried=empty` | `state=1, items=46, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot36=count4/item1, carried=count4/item1` | `state=3, items=46, slot36=count4/item1, carried=count4/item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1116` | `1524` |

Her iki geçerli console item replacement işlemi başarıyla tamamlandı. Server
8 taşlık hotbar stack'inin 4'ünü carried item yaptı, slot 36'da 4 bıraktı ve
eski state-id nedeniyle state `3` tam content düzeltmesi gönderdi. Vanilla
logunda protocol error, action warning veya timeout gözlenmedi; iki oturumun
karşılaştırılan inventory çıktıları aynıdır.

İki ara koşul (`InvHalf2`, `InvHalf3`) item replacement komutunun probe click
beklemesine çok yakın işlenmesi nedeniyle geçerli parity koşulu olarak alınmadı.
Bu koşullarda item değişiminin ve click düzeltmesinin state sırası karıştı;
sonuçlar hazırlık zamanlamasının kontrol edilmesi gerektiğini gösterdi.

## Sonuç ve açık kapsam

İki bağımsız geçerli vanilla oturumu, secondary pickup state-id uyuşmazlığında
yarım-stack işleminin uygulanıp slot ve cursor içeriğinin authoritative full
resync ile düzeltildiğini doğrular. Sözleşme ayrıntıları [survival inventory
secondary pickup state correction contract belgesindedir](survival-inventory-secondary-pickup-state-correction-contract.md).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, diğer pickup
stack koşulları, throw, drag, double-click, stack sınırı ve özel menu akışları
açık kalır.
