# P13 — Vanilla survival inventory empty drag cancellation state-id correction doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Bu senaryoda item hazırlığı
yapılmadı; başlangıç player inventory'sinin boş content'i doğrudan kullanıldı.

```text
cargo run --offline --release --example survival_inventory_drag_empty_cancel_state_correction_probe -- 127.0.0.1:25566 InvDrag1
cargo run --offline --release --example survival_inventory_drag_empty_cancel_state_correction_probe -- 127.0.0.1:25566 InvDrag2
```

Probe, player inventory menu state `1` iken slot `-999`, button `0` ve
`QUICK_CRAFT` input id `5` ile drag başlangıcı yollar; state-id'yi `0`,
changed-slot map'ini ve carried hash'i boş tutar. Carried boş olduğu için
vanilla quick-craft durumunu sıfırlar ve stale state nedeniyle no-op full resync
yapar.

## Sonuçlar

| Gözlem | InvDrag1 | InvDrag2 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / slot / button / input id | `0 / -999 / 0 / 5` | `0 / -999 / 0 / 5` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| container slot observations | `0` | `0` |
| authoritative full resync content | `state=2, items=46, all empty, carried=empty` | `state=2, items=46, all empty, carried=empty` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `396` | `360` |

Her iki vanilla oturumunda quick-craft başlangıcı boş carried nedeniyle iptal
edildi; slot veya cursor item'i oluşmadı ve stale state-id nedeniyle state `2`
full content düzeltmesi gönderildi. Vanilla logunda protocol error, action
warning veya timeout gözlenmedi; karşılaştırılan inventory çıktıları aynıdır.

## Sonuç ve açık kapsam

İki bağımsız vanilla oturumu, empty-carried `QUICK_CRAFT` başlangıcının no-op
kalıp authoritative full resync ile state `2`'ye düzeltildiğini doğrular.
Sözleşme ayrıntıları [survival inventory empty drag cancellation state correction
contract belgesindedir](survival-inventory-drag-empty-cancel-state-correction-contract.md).

Carried item ile gerçek drag dağıtımı, drag header/add/end sırası, doğru state-id
ile client'ın hashed prediction'ı, double-click, stack sınırı ve özel menu
akışları açık kalır.
