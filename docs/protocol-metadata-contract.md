# P06/P07 — Protocol phase ve packet flow metadata sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ConnectionProtocol` enum'u beş phase'i kaynak sırasıyla `HANDSHAKING`, `PLAY`, `STATUS`, `LOGIN` ve `CONFIGURATION` olarak taşır. Her phase'in kısa kimliği sırasıyla `handshake`, `play`, `status`, `login` ve `configuration` değeridir; enum ordinal'i kaynak sırasını izler.

`PacketFlow` iki yön taşır: `SERVERBOUND` kimliği `serverbound`, `CLIENTBOUND` kimliği `clientbound` değeridir. `getOpposite` serverbound için clientbound, clientbound için serverbound döndürür. Yön enum ordinal'leri de bu kaynak sırasına göre 0 ve 1'dir.

Rust `ConnectionProtocol` ve `PacketFlow` değerlerini aynı sırada, aynı kimliklerle ve aynı opposite davranışıyla taşır. Bu sözleşme metadata değerleriyle sınırlıdır; packet id dispatch kayıtları, frame pipeline, socket connection ve handshake'ten play'e gerçek phase geçişi sonraki P06/P07 kapsamıdır.
