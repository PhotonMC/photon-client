# P01.b — Vec3 isFinite doğrulaması

`src/math.rs`, Java `Vec3.isFinite()` metodunu karşılar. Rust adaptörü üç f64 bileşenini sonluluk için sorgular; Java oracle doğrudan `Vec3.isFinite` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1221 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1221/1221** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu sınırlar, signed zero, pozitif/negatif Infinity, payload taşıyan NaN ve karışık özel değerler karşılaştırıldı.
- Birleşik altı aile corpus'u **1723/1723** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1723 senaryoluk Java referansını yeniden doğruladı.

Math referansı `df8b0fd452c7f12175fe9ced3c534c1f64bdb7df75cc28529becd0eb8ab97b3c`, birleşik referans `86380f621c6f188dddfa1e68a94057a8ada6d638b9f5c04d3a2c9682b1ca75cf` kimliğindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.isFinite` sonucunun boolean çıktısıyla sınırlıdır. Java `Double.isFinite` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
