# P01.b — Vec3 toString doğrulaması

`src/math.rs`, Java `Vec3.toString()` davranışını `Vec3::to_string_java` ile karşılar. Java oracle, sonucu doğrudan Java metodundan alır; double biçimlendirme yordamı AABB metin yolu ile paylaşılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1335 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1335/1335** math senaryosunda eşleşti.
- Altı yeni senaryoda temel biçim, bilimsel gösterim eşikleri, signed zero, Infinity ve farklı NaN bit desenleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1837/1837** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1837 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8956f11f5ab0ba4c81a6aa2c11af8623a498bbb78a550a1b013695afc5029da2`, birleşik referans `f90c38c2c4e4eb83aec92229706d7f6b47bc90a128aa4908ed60897e8279a0d9` kimliğindedir. Math karşılaştırma raporları `compare-0dy1p_jx` (debug) ve `compare-fdzx3gij` (release); birleşik raporlar `compare-gridp_gxu` (debug) ve `compare-u8zwfn4r` (release), Java doğrulaması `verify-yhwxvq8l` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 metninin Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. Java `Double.toString` için daha geniş tüm bit desenleri matrisi ve diğer sınıfların metin biçimleri ayrı dilimlerdir.
