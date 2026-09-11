# P12 — Blok eylemi sequence ACK sözleşmesi

Bu dilim, ilk dünya yükleme akışından sonra vanilla server'a zararsız bir `ABORT_DESTROY_BLOCK` packet'i gönderir ve sequence ACK'ini okur. Amaç, block action wire alanlarının, oyun oturumundaki serverbound packet ID'sinin ve server'ın `ClientboundBlockChangedAckPacket` cevabının çalıştığını sınamaktır.

## Java referansı

`ServerboundPlayerActionPacket` action enum/BlockPos/direction/sequence sırasıyla, `ServerGamePacketListenerImpl.handlePlayerAction` içindeki loaded kontrolü ve `ackBlockChangesUpTo` çağrısıyla, server tick'inde `ClientboundBlockChangedAckPacket` gönderimiyle birlikte incelendi. `FriendlyByteBuf.writeEnum`, `writeBlockPos` ve Direction'ın `DOWN` wire ID'si de kontrol edildi. Orijinal kaynak depoya kopyalanmadı.

## Probe girdisi ve akışı

- Action: `ABORT_DESTROY_BLOCK` enum ordinal `1`
- Block position: `BlockPos.ZERO`, packed long `0`
- Direction: `DOWN`, ID `0`
- Sequence: `1`

Bu sentinel hedef menzil dışında olduğu için block break state'i veya geçici dünya içeriği değişmez. Java server yine sequence'i ACK kuyruğuna alır; sonraki tick'te `ClientboundBlockChangedAckPacket(sequence=1)` gönderir. Rust trace gönderilen action sayısını ve ACK sequence listesini kaydeder.

## Sınır

Bu dilim hit test, reach, gerçek block state değişimi, kazma ilerlemesi/iptali, placement, item use, prediction/rollback ve client world cache uygulamaz. ACK alınması oynanabilir istemci veya Java action davranışının tamamı değildir.
