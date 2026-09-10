# P06.d — Ortak ping/pong ve keep-alive paket sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundPingPacket` ile `ServerboundPongPacket` bir signed 32-bit integer taşır. `ClientboundKeepAlivePacket` ile `ServerboundKeepAlivePacket` bir signed 64-bit integer taşır. Her dört packet alanını FriendlyByteBuf üzerinde Java primitive `writeInt`/`readInt` veya `writeLong`/`readLong` ile big-endian sırada kodlar.

Yazma yalnızca ilgili sabit genişlikteki baytları ekler ve mevcut prefix'i korur. Okuma alan genişliğinden kısa buffer'da underflow üretir ve primitive okuma atomic olduğu için input tüketmez; yeterli buffer'da yalnızca alanı tüketir ve trailing veriyi bırakır. Packet'ler için ek aralık veya değer doğrulaması yoktur.

Rust tarafı dört packet'i ortak public alanı `id` ve `new`/`encode`/`decode` API'siyle taşır. Bu dilim packet payload, byte sırası ve buffer tüketimiyle sınırlıdır; keep-alive yanıt zamanlaması, disconnect/reconnect ve gerçek connection state P06.d entegrasyon kapsamıdır.
