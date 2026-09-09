# P01.b — Vec3 isFinite doğrulaması

`src/math.rs`, Java `Vec3.isFinite()` metodunu karşılar. Rust adaptörü üç f64 bileşenini sonluluk için sorgular; Java oracle doğrudan `Vec3.isFinite` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 905 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **905/905** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu sınırlar, signed zero, pozitif/negatif Infinity, payload taşıyan NaN ve karışık özel değerler karşılaştırıldı.
- Birleşik altı aile corpus'u **1387/1387** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1387 senaryoluk Java referansını yeniden doğruladı.

Math referansı `847816d7bf7d2e0e9cecc67b5d7010685842d00cd4860fb3cf9447d37a9fe8a9`, birleşik referans `a2de3a61075984570c78972cf64abf406bbb6ee6d60136396680e14b82219664` kimliğindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.isFinite` sonucunun boolean çıktısıyla sınırlıdır. Java `Double.isFinite` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
