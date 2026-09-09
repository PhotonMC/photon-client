# P01.b — Vec3 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec3.equals(Object)` ve `Vec3.hashCode()` davranışlarını karşılar. Java oracle, karşılaştırmayı ve iki hash sonucunu doğrudan Java metodlarından alır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1007 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1007/1007** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu değerler, farklı bileşen, signed zero, farklı NaN payload'ları, Infinity ve özel değerlerin birlikte karşılaştırılması ölçüldü.
- Birleşik altı aile corpus'u **1509/1509** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1509 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7b8b9ea0f5717fc78d7092f1d3563134bfdae91e1c70cf411bb3269f1ad58d4d`, birleşik referans `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a` kimliğindedir. Math karşılaştırma raporları `compare-upfzoh9c` (debug) ve `compare-czpd1pgo` (release); birleşik raporlar `compare-_3h0xf61` (debug) ve `compare-a2h2wcm2` (release), Java doğrulaması `verify-19j4rwux` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 eşleştirmesinde `equals` boolean'ı ile `hashCode` decimal string sonuçlarının Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. `Vec3.toString()` ve diğer nesne biçimlendirme davranışları ayrı dilimlerdir.
