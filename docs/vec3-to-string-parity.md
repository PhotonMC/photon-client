# P01.b — Vec3 toString doğrulaması

`src/math.rs`, Java `Vec3.toString()` davranışını `Vec3::to_string_java` ile karşılar. Java oracle, sonucu doğrudan Java metodundan alır; double biçimlendirme yordamı AABB metin yolu ile paylaşılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1301 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1301/1301** math senaryosunda eşleşti.
- Altı yeni senaryoda temel biçim, bilimsel gösterim eşikleri, signed zero, Infinity ve farklı NaN bit desenleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1803/1803** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1803 senaryoluk Java referansını yeniden doğruladı.

Math referansı `a7681fdbc73c8faa861a3c0f74be7f2286257b8c494b6f1d3c85598ad65f6973`, birleşik referans `e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935` kimliğindedir. Math karşılaştırma raporları `compare-zq98bs3k` (debug) ve `compare-zgg863t0` (release); birleşik raporlar `compare-yz31bmcf` (debug) ve `compare-90dzj4qp` (release), Java doğrulaması `verify-l29xakrv` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 metninin Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. Java `Double.toString` için daha geniş tüm bit desenleri matrisi ve diğer sınıfların metin biçimleri ayrı dilimlerdir.
