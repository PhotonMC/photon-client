# P01.b — Vec3 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec3.equals(Object)` ve `Vec3.hashCode()` davranışlarını karşılar. Java oracle, karşılaştırmayı ve iki hash sonucunu doğrudan Java metodlarından alır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 869 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **869/869** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu değerler, farklı bileşen, signed zero, farklı NaN payload'ları, Infinity ve özel değerlerin birlikte karşılaştırılması ölçüldü.
- Birleşik altı aile corpus'u **1323/1323** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1323 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6d26ec313384e0e1de60baf56cec198f36f3856e1de1e7580da700e48ce92ca1`, birleşik referans `b5e20f42ea20b904f73ce2c8cd50b3c271f9e5405ad39815c2cbd61da6b7fb92` kimliğindedir. Math karşılaştırma raporları `compare-tq2_z6ii` (debug) ve `compare-889giagp` (release); birleşik raporlar `compare-sc8wywfj` (debug) ve `compare-wvhin971` (release), Java doğrulaması `verify-z4jw4eod` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 eşleştirmesinde `equals` boolean'ı ile `hashCode` decimal string sonuçlarının Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. `Vec3.toString()` ve diğer nesne biçimlendirme davranışları ayrı dilimlerdir.
