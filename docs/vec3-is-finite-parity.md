# P01.b — Vec3 isFinite doğrulaması

`src/math.rs`, Java `Vec3.isFinite()` metodunu karşılar. Rust adaptörü üç f64 bileşenini sonluluk için sorgular; Java oracle doğrudan `Vec3.isFinite` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1608 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1608/1608** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu sınırlar, signed zero, pozitif/negatif Infinity, payload taşıyan NaN ve karışık özel değerler karşılaştırıldı.
- Birleşik altı aile corpus'u **2110/2110** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2110 senaryoluk Java referansını yeniden doğruladı.

Math referansı `10c179a43201ead1d005dbd3e8f1deb7d1b4d5928bd228214792c84e07ef2b41`, birleşik referans `bb3698ca7b34e45053dd8865ed3b754080b1fb665c802937b38ccb0216ecede3` kimliğindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.isFinite` sonucunun boolean çıktısıyla sınırlıdır. Java `Double.isFinite` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
