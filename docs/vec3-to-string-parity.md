# P01.b — Vec3 toString doğrulaması

`src/math.rs`, Java `Vec3.toString()` davranışını `Vec3::to_string_java` ile karşılar. Java oracle, sonucu doğrudan Java metodundan alır; double biçimlendirme yordamı AABB metin yolu ile paylaşılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1025 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1025/1025** math senaryosunda eşleşti.
- Altı yeni senaryoda temel biçim, bilimsel gösterim eşikleri, signed zero, Infinity ve farklı NaN bit desenleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1527/1527** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1527 senaryoluk Java referansını yeniden doğruladı.

Math referansı `2c79e2aaba1154782177df83cd39488c1c9a365c76e1971733e837c447fe9fa3`, birleşik referans `74f88dfa13fb755a92b838953181016d177ef3a05b113b5fa8ef1f3735031acc` kimliğindedir. Math karşılaştırma raporları `compare-0dsoh6yn` (debug) ve `compare-wdh5yokb` (release); birleşik raporlar `compare-7wace8y9` (debug) ve `compare-ohni39xi` (release), Java doğrulaması `verify-a7iqdyvs` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 metninin Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. Java `Double.toString` için daha geniş tüm bit desenleri matrisi ve diğer sınıfların metin biçimleri ayrı dilimlerdir.
