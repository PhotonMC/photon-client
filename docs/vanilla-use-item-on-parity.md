# P12 — Vanilla 26.2 boş elle `use_item_on` doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example use_item_on_probe -- 127.0.0.1:25566 RustUse1
cargo run --offline --example use_item_on_probe -- 127.0.0.1:25566 RustUse2
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustUse1 | RustUse2 |
| --- | ---: | ---: |
| play login | `true` | `true` |
| player loaded sent | `true` | `true` |
| use item on packets sent | 1 | 1 |
| block changed ACK sequences | `[4]` | `[4]` |
| block update observations | `[{x:26,y:62,z:-42,state_id:9},{x:26,y:63,z:-42,state_id:0}]` | `[{x:27,y:70,z:-36,state_id:10},{x:27,y:71,z:-36,state_id:0}]` |
| chunk packet | 21 | 21 |
| decoded chunk | 21 | 21 |
| chunk batch finished | 5 | 5 |
| received packet | 406 | 337 |

İlk koşunun spawn konumu `(26.5,63.0,-41.5)`, ikinci koşunun spawn konumu `(27.5,71.0,-35.5)` idi; her iki hedef de oyuncunun bir blok altındaydı. Target ve `UP` komşusu için dönen state ID'leri, seed `776` dünyasındaki o anki yüzey bloklarından geldi ve boş el nedeniyle değişmedi. Server logunda protocol error, timeout veya `UseItemOnPacket` rejection warning oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server üzerinde boş elle `use_item_on` packet alanlarını, reach/hit-location kabulünü, iki block update dönüşünü ve ACK'i doğrular. Item stack codec'i, creative slot, gerçek placement, item use sonucu, client prediction/rollback, inventory ve görsel/ses feedback'i açık kalır.

Sözleşme ayrıntıları [use item on contract belgesindedir](use-item-on-contract.md). Creative block break [ayrı kayıttadır](vanilla-block-break-parity.md); placement için item/inventory ön koşulu ayrıca hazırlanacaktır.
