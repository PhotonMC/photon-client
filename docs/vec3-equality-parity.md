# P01.b — Vec3 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec3.equals(Object)` ve `Vec3.hashCode()` davranışlarını karşılar. Java oracle, karşılaştırmayı ve iki hash sonucunu doğrudan Java metodlarından alır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 929 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **929/929** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu değerler, farklı bileşen, signed zero, farklı NaN payload'ları, Infinity ve özel değerlerin birlikte karşılaştırılması ölçüldü.
- Birleşik altı aile corpus'u **1431/1431** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1431 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8fdf229fdfa07bea578ab4c7a2dd8d338928e403b9c3475b341fcc20842f0def`, birleşik referans `e01fd95590d0e47ec886dfd327be291cda2db519ff726b58b32f5fa0a8e99c88` kimliğindedir. Math karşılaştırma raporları `compare-eiqs7rub` (debug) ve `compare-t3kloypj` (release); birleşik raporlar `compare-wfqzv7gl` (debug) ve `compare-_to6641m` (release), Java doğrulaması `verify-oni_ax6_` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 eşleştirmesinde `equals` boolean'ı ile `hashCode` decimal string sonuçlarının Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. `Vec3.toString()` ve diğer nesne biçimlendirme davranışları ayrı dilimlerdir.
