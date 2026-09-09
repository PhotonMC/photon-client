# P01.b — Vec3 toString doğrulaması

`src/math.rs`, Java `Vec3.toString()` davranışını `Vec3::to_string_java` ile karşılar. Java oracle, sonucu doğrudan Java metodundan alır; double biçimlendirme yordamı AABB metin yolu ile paylaşılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 947 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **947/947** math senaryosunda eşleşti.
- Altı yeni senaryoda temel biçim, bilimsel gösterim eşikleri, signed zero, Infinity ve farklı NaN bit desenleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1449/1449** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1449 senaryoluk Java referansını yeniden doğruladı.

Math referansı `173698891dc93629abbcd5ca3642ca5af04a5ddb10ffcbaa0610cf03306c320e`, birleşik referans `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363` kimliğindedir. Math karşılaştırma raporları `compare-auuk33g2` (debug) ve `compare-w_2b2455` (release); birleşik raporlar `compare-tjsmsqxw` (debug) ve `compare-sks6l8sa` (release), Java doğrulaması `verify-egp3gvx_` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 metninin Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. Java `Double.toString` için daha geniş tüm bit desenleri matrisi ve diğer sınıfların metin biçimleri ayrı dilimlerdir.
