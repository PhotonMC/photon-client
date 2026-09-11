# P06.a — Status request ve ping/pong sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ServerboundStatusRequestPacket.STREAM_CODEC` `StreamCodec.unit(INSTANCE)` olduğu için yazmada hiçbir bayt üretmez; okumada input'u incelemeden aynı singleton'ı döndürür. Prefix ve trailing baytlar codec'e ait değildir ve korunur.

`ServerboundPingRequestPacket` ile `ClientboundPongResponsePacket` bir signed Java `long` taşır. Değer sekiz bayt big-endian olarak yazılır ve okunur. Rust API'si `i64` kullanır; sekiz bayttan kısa input `BufferUnderflow` üretir ve atomic `readLong` davranışıyla input'u ilerletmez.

Bu dilim packet payload codec'leriyle sınırlıdır. VarInt21 frame sınırı, status JSON cevabının `ServerStatus` codec'i ve tam protocol state transition ayrı dilimlerdir. Ham status response framing'i ile socket üzerindeki ping echo'su [ayrı bağlantı sözleşmesinde](status-connection-contract.md) ölçülür.
