# P13 — Vanilla survival inventory throw-all state-id correction doğrulaması

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
cargo run --offline --release --example survival_inventory_throw_all_state_correction_probe -- 127.0.0.1:25566 InvThrowAll3
cargo run --offline --release --example survival_inventory_throw_all_state_correction_probe -- 127.0.0.1:25566 InvThrowAll4

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken slot 36'ya button `1` ve `THROW`
input id `4` ile tıklama yollar; state-id'yi `0`, changed-slot map'ini ve
carried hash'i boş tutar. Böylece server'ın 8 taşlık stack'i tamamen düşürüp
slotu boşaltması ve full resync yapması ayırt edilir.

## Sonuçlar

| Gözlem | InvThrowAll3 | InvThrowAll4 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / button / input id | `0 / 1 / 4` | `0 / 1 / 4` |
| initial container content | `state=1, items=46, slot36=empty, carried=empty` | `state=1, items=46, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot36=empty, carried=empty` | `state=3, items=46, slot36=empty, carried=empty` |
| dropped-item add entity | `1 × entity_type_id=71 near player` | `1 × entity_type_id=71 near player` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `4743` | `3837` |

Her iki geçerli console item replacement işlemi başarıyla tamamlandı. Server
8 taşlık stack'in tamamını tek drop olarak düşürdü, slot 36'yı boşalttı ve eski
state-id nedeniyle state `3` tam content düzeltmesi gönderdi. Her iki oturumda
oyuncu konumuna yakın tek `entity_type_id=71` add-entity gözlemi alındı. Probe
add-entity metadata'sındaki gerçek item stack component'lerini decode etmedi;
bu parity kaydındaki inventory sonucu slot content üzerinden doğrulandı.

Vanilla logunda protocol error, action warning veya timeout gözlenmedi; iki
oturumun karşılaştırılan inventory çıktıları aynıdır.

## Sonuç ve açık kapsam

İki bağımsız geçerli vanilla oturumu, throw-all state-id uyuşmazlığında tüm
stack drop işleminin uygulanıp slot ve cursor içeriğinin authoritative full
resync ile düzeltildiğini doğrular. Sözleşme ayrıntıları [survival inventory
throw-all state correction contract belgesindedir](survival-inventory-throw-all-state-correction-contract.md).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, drag,
double-click, stack sınırı ve özel menu akışları açık kalır.
