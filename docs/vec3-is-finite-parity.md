# P01.b — Vec3 isFinite doğrulaması

`src/math.rs`, Java `Vec3.isFinite()` metodunu karşılar. Rust adaptörü üç f64 bileşenini sonluluk için sorgular; Java oracle doğrudan `Vec3.isFinite` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1019 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1019/1019** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu sınırlar, signed zero, pozitif/negatif Infinity, payload taşıyan NaN ve karışık özel değerler karşılaştırıldı.
- Birleşik altı aile corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1521 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5df2c7aa290a2d3deb2e318553a41a27dbe6b08b22e1d31b68362eedc36e2108`, birleşik referans `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406` kimliğindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.isFinite` sonucunun boolean çıktısıyla sınırlıdır. Java `Double.isFinite` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
