# P01.b — Vec3 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec3.equals(Object)` ve `Vec3.hashCode()` davranışlarını karşılar. Java oracle, karşılaştırmayı ve iki hash sonucunu doğrudan Java metodlarından alır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 917 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **917/917** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu değerler, farklı bileşen, signed zero, farklı NaN payload'ları, Infinity ve özel değerlerin birlikte karşılaştırılması ölçüldü.
- Birleşik altı aile corpus'u **1419/1419** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1419 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7c835b893c95d986797a072a7b69eba5f1f31dad85d36d5bd51ba6394a221696`, birleşik referans `cfb0f4a3444412c75186c0d8b209a59632b9d62debcc435aa548bf4c364d8656` kimliğindedir. Math karşılaştırma raporları `compare-dvzelshx` (debug) ve `compare-sy72qqem` (release); birleşik raporlar `compare-ng1mczig` (debug) ve `compare-7brjjbpz` (release), Java doğrulaması `verify-3oltsdwp` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 eşleştirmesinde `equals` boolean'ı ile `hashCode` decimal string sonuçlarının Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. `Vec3.toString()` ve diğer nesne biçimlendirme davranışları ayrı dilimlerdir.
