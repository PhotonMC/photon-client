# P01.b — Vec3 isFinite doğrulaması

`src/math.rs`, Java `Vec3.isFinite()` metodunu karşılar. Rust adaptörü üç f64 bileşenini sonluluk için sorgular; Java oracle doğrudan `Vec3.isFinite` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 869 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **869/869** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu sınırlar, signed zero, pozitif/negatif Infinity, payload taşıyan NaN ve karışık özel değerler karşılaştırıldı.
- Birleşik altı aile corpus'u **1323/1323** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1323 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6d26ec313384e0e1de60baf56cec198f36f3856e1de1e7580da700e48ce92ca1`, birleşik referans `b5e20f42ea20b904f73ce2c8cd50b3c271f9e5405ad39815c2cbd61da6b7fb92` kimliğindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.isFinite` sonucunun boolean çıktısıyla sınırlıdır. Java `Double.isFinite` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
