# P07.a — Enabled features packet sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundUpdateEnabledFeaturesPacket` payload'ı önce feature sayısını VarInt, sonra her feature'ı `FriendlyByteBuf` üzerindeki varsayılan 32767 UTF-16 birimlik UTF-8 alanı olarak taşır.

Her feature `Identifier.parse` ile çözülür. İlk `:` namespace/path ayracıdır; ayracın başında veya hiç bulunmamasında namespace `minecraft` olur. Namespace yalnızca küçük ASCII harf, rakam, `_`, `-`, `.` içerir ve `..` olamaz; path aynı karakterlere ek olarak `/` kabul eder. Boş path Java tarafından geçerlidir. Geçersiz identifier decode sırasında decoder hatasıdır.

Packet alanı Java `Set<Identifier>` olduğu için duplicate feature'lar tekilleşir. Java encode yolu varsayılan `HashSet` iterasyon sırasını, decode yolu ise count ile oluşturulan `HashSet` sırasını kullanır; bucket kapasitesi bu iki yolu ayırabilir. Sabit alan genişliğinden kısa string/body okumaları Java UTF-8 codec'inin tüketim kuralını korur: uzunluk prefix'i okunur, gövde eksikse gövde tüketilmeden decoder hatası verilir. Count VarInt'i tüketilir; negatif count decoder hatasıdır.

Rust tarafı Identifier doğrulamasını, Java String hash/spread hash tabanlı HashSet sırasını, duplicate tekilleştirmeyi ve collection'ın kısmi okuma/yazma davranışını taşır. Bu dilim yalnızca feature payload'ı kapsar; `FeatureFlagRegistry.fromNames`, registry biriktirme ve configuration listener state geçişi ayrı kapsamdır.
