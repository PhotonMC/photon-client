# P01.b — Vec3 toString doğrulaması

`src/math.rs`, Java `Vec3.toString()` davranışını `Vec3::to_string_java` ile karşılar. Java oracle, sonucu doğrudan Java metodundan alır; double biçimlendirme yordamı AABB metin yolu ile paylaşılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 893 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **893/893** math senaryosunda eşleşti.
- Altı yeni senaryoda temel biçim, bilimsel gösterim eşikleri, signed zero, Infinity ve farklı NaN bit desenleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1360/1360** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1360 senaryoluk Java referansını yeniden doğruladı.

Math referansı `14e060de3e2dfd133faf305681288b099fffc157d99d2140efcced661186142a`, birleşik referans `560602ef9264eceeefea1f7e04180a821c7e67a9bd644f85964cbe1229e4e713` kimliğindedir. Math karşılaştırma raporları `compare-gnxkpf0j` (debug) ve `compare-u696_8k6` (release); birleşik raporlar `compare-khi93grl` (debug) ve `compare-qhum5wxw` (release), Java doğrulaması `verify-ttb2wdcj` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 metninin Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. Java `Double.toString` için daha geniş tüm bit desenleri matrisi ve diğer sınıfların metin biçimleri ayrı dilimlerdir.
