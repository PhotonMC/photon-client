# P07 — ClientInformation ve configuration unit packet sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ServerboundClientInformationPacket` alanları şu sıradadır: UTF-8 language, byte view distance, chat visibility ordinal VarInt, chat colors boolean, byte model customisation, main hand ordinal VarInt, text filtering boolean, allows listing boolean ve particle status ordinal VarInt.

Language okumada 16 UTF-16 birimi sınırına, yazmada `writeUtf(String)` varsayılanı olan 32767 sınıra tabidir. View distance Java signed byte olarak okunur; model customisation unsigned byte olarak okunur. Yazmada iki değer de Java `writeByte` gibi düşük sekiz biti taşır. Enum okumada ordinal doğrudan dizi indeksidir; geçersiz ordinal decoder hatasıdır. Boolean alanlar sıfır dışı byte'ı true kabul eder.

`ServerboundFinishConfigurationPacket`, `ClientboundFinishConfigurationPacket` ve `ServerboundAcceptCodeOfConductPacket` unit codec kullanır. Yazma sıfır byte ekler; okuma input'u ilerletmeden singleton packet döndürür.

Rust tarafı enum'ları açık ordinal değerleriyle, primitive alanları Java byte genişlikleriyle ve unit packet'leri singleton değerlerle taşır. Bu dilim payload codec ve alan tüketimiyle sınırlıdır; configuration listener/state transition, registry verisi ve play phase ayrı kapsamdır.
