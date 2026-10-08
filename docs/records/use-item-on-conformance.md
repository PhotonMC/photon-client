# P12 — Boş elle `use_item_on` sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, ilk dünya yüklemesinden sonra oyuncunun spawn olduğu konumu destekleyen bloğa boş main hand ile `use_item_on` gönderir. Amaç, gerçek `BlockHitResult` wire alanlarını, hedef menzil doğrulamasını, sequence ACK'ini ve vanilla server'ın etkileşim sonrası gönderdiği iki block update'i ölçmektir.

## Java referansı

`ServerboundUseItemOnPacket` hand, `BlockHitResult` ve sequence alanlarını taşır. `FriendlyByteBuf.writeBlockHitResult` packed `BlockPos`, face enum'u, block-relative click `float` değerleri ve iki boolean'ı bu sırada yazar. `ServerGamePacketListenerImpl.handleUseItemOn` loaded kontrolünden sonra sequence'i ACK kuyruğuna alır, menzil ve hit-location sınırlarını kontrol eder ve geçerli akışın sonunda hedef ile face yönündeki komşu için `ClientboundBlockUpdatePacket` gönderir.

## Probe girdisi ve akışı

- Hand: `MAIN_HAND`, enum ordinal `0`; server tarafındaki el boştur
- Hedef: spawn player konumunun bir blok altındaki Java floor hedefi
- Face: `UP`, enum ID `1`
- Click: block-relative `(0.5, 1.0, 0.5)` f32
- `inside=false`, `worldBorderHit=false`
- Sequence: `4`

İlk `player_loaded` sonrasında Rust tek packet gönderir. Boş el block state'i değiştirmez; vanilla yine hedef blok ve `UP` komşusu için block update'leri, ayrıca ACK `[4]` döndürür. Bu probe placement veya item use sonucu değildir.

## Sınır

Bu dilim item stack/component, creative hotbar doldurma, gerçek block placement, block/entity hit ray'i, prediction/rollback ve görsel/ses feedback'ini uygulamaz. Placement için önce serverbound creative slot/item stack codec'i ayrı doğrulanmalıdır.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

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

Sözleşme ayrıntıları [use item on contract belgesindedir](#sözleşme). Creative block break [ayrı kayıttadır](block-break-conformance.md); placement için item/inventory ön koşulu ayrıca hazırlanacaktır.
