# P01.b — Vec3 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec3.equals(Object)` ve `Vec3.hashCode()` davranışlarını karşılar. Java oracle, karşılaştırmayı ve iki hash sonucunu doğrudan Java metodlarından alır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 887 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **887/887** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu değerler, farklı bileşen, signed zero, farklı NaN payload'ları, Infinity ve özel değerlerin birlikte karşılaştırılması ölçüldü.
- Birleşik altı aile corpus'u **1341/1341** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1341 senaryoluk Java referansını yeniden doğruladı.

Math referansı `676119520815e68dd9f4f120c8201463ce27aa4921910181223d0cdb38c939ec`, birleşik referans `971b4d305a95fb8e346f40a2edb704ac7f42751d32b8193786c4eb6aae63d1fb` kimliğindedir. Math karşılaştırma raporları `compare-d1bitq7h` (debug) ve `compare-ldlepu8u` (release); birleşik raporlar `compare-kuzkqdbl` (debug) ve `compare-n3x2kj9p` (release), Java doğrulaması `verify-jbdhw9mf` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 eşleştirmesinde `equals` boolean'ı ile `hashCode` decimal string sonuçlarının Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. `Vec3.toString()` ve diğer nesne biçimlendirme davranışları ayrı dilimlerdir.
