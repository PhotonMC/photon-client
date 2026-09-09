# P01.b — Vec3 toString doğrulaması

`src/math.rs`, Java `Vec3.toString()` davranışını `Vec3::to_string_java` ile karşılar. Java oracle, sonucu doğrudan Java metodundan alır; double biçimlendirme yordamı AABB metin yolu ile paylaşılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 977 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **977/977** math senaryosunda eşleşti.
- Altı yeni senaryoda temel biçim, bilimsel gösterim eşikleri, signed zero, Infinity ve farklı NaN bit desenleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1479/1479** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1479 senaryoluk Java referansını yeniden doğruladı.

Math referansı `fc20607e30d3e5c68224006103ae9dd1fce364a83ec76ae93deae73d073194b8`, birleşik referans `65cbf0b19b2c96be41cf3bbad7266d1e118047e47e641f45d1473f7a741de148` kimliğindedir. Math karşılaştırma raporları `compare-89j99zq5` (debug) ve `compare-8bts9s4m` (release); birleşik raporlar `compare-88a_ixm3` (debug) ve `compare-sk_17ilr` (release), Java doğrulaması `verify-yeb5s_0e` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 metninin Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. Java `Double.toString` için daha geniş tüm bit desenleri matrisi ve diğer sınıfların metin biçimleri ayrı dilimlerdir.
