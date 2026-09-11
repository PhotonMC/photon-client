# P12 — Boş elle `use_item_on` sözleşmesi

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
