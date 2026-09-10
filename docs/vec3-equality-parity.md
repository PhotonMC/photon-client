# P01.b — Vec3 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec3.equals(Object)` ve `Vec3.hashCode()` davranışlarını karşılar. Java oracle, karşılaştırmayı ve iki hash sonucunu doğrudan Java metodlarından alır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1653 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1653/1653** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu değerler, farklı bileşen, signed zero, farklı NaN payload'ları, Infinity ve özel değerlerin birlikte karşılaştırılması ölçüldü.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2155 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`, birleşik referans `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e` kimliğindedir. Math karşılaştırma raporları `compare-ceusm4y4` (debug) ve `compare-hconpy1a` (release); birleşik raporlar `compare-rhjx0z7e` (debug) ve `compare-jtl411eu` (release), Java doğrulaması `verify-u5d5zd4c` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 eşleştirmesinde `equals` boolean'ı ile `hashCode` decimal string sonuçlarının Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. `Vec3.toString()` ve diğer nesne biçimlendirme davranışları ayrı dilimlerdir.
