# P01.b — Vec3 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec3.equals(Object)` ve `Vec3.hashCode()` davranışlarını karşılar. Java oracle, karşılaştırmayı ve iki hash sonucunu doğrudan Java metodlarından alır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 935 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **935/935** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu değerler, farklı bileşen, signed zero, farklı NaN payload'ları, Infinity ve özel değerlerin birlikte karşılaştırılması ölçüldü.
- Birleşik altı aile corpus'u **1437/1437** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1437 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b5430468b5bb806bec96f53c626329209858940d1a04da9bbfbedb16ab6b2dc0`, birleşik referans `2a0e476c9429ade902407539ef06621159d2eed0b5010504338eecbf1e77e14a` kimliğindedir. Math karşılaştırma raporları `compare-96_s862r` (debug) ve `compare-cbwrzmb5` (release); birleşik raporlar `compare-fn36ocz_` (debug) ve `compare-a_hggp11` (release), Java doğrulaması `verify-k4za48of` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 eşleştirmesinde `equals` boolean'ı ile `hashCode` decimal string sonuçlarının Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. `Vec3.toString()` ve diğer nesne biçimlendirme davranışları ayrı dilimlerdir.
