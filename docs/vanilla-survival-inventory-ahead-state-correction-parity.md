# P13 — Vanilla survival inventory ahead state-id correction doğrulaması

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
cargo run --offline --release --example survival_inventory_ahead_state_correction_probe -- 127.0.0.1:25566 InvAhead2
cargo run --offline --release --example survival_inventory_ahead_state_correction_probe -- 127.0.0.1:25566 InvAhead3
```

Probe, item hazırlığı sonrası menu state `2` iken state-id `99`, slot `36`,
button `0`, `PICKUP` input id `0`, boş changed-slot map'i ve empty carried
hash'i ile tek click gönderdi.

## Sonuçlar

| Gözlem | InvAhead2 | InvAhead3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `99 / 36 / 0 / 0` | `99 / 36 / 0 / 0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| authoritative full resync | `state=3, slot36=empty, carried=count8,item1` | `state=3, slot36=empty, carried=count8,item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2829` | `1566` |

Her iki vanilla oturumunda state-id `99`, menu state `2` ile eşleşmediği halde
pickup click'i uygulandı ve state `3` full content düzeltmesiyle slot boşalıp
8 taş cursor'a taşındı. Server logunda koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, menu state-id'sinden büyük bir
serverbound container state-id'nin de click'i uygulayıp authoritative full
resync'e girdiğini doğrular. Sözleşme ayrıntıları [survival inventory ahead
state-id correction contract belgesindedir](survival-inventory-ahead-state-correction-contract.md).

Non-empty changed-slot map'i, non-empty carried hash'i, diğer input/slot
varyantları, doğru state-id ile client'ın hashed prediction'ı ve özel menu
akışları açık kalır.
