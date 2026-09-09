# P01.b — Vec3 with doğrulaması

`src/math.rs`, Java `Vec3.with(Direction.Axis, double)` metodunu karşılar. Rust adaptörü seçilen bileşeni verilen f64 değere taşır ve diğer bileşen bitlerini korur; Java oracle doğrudan `Vec3.with` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 883 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **883/883** math senaryosunda eşleşti.
- Beş yeni senaryoda X/Y/Z seçimi, değiştirilmemiş bileşenlerin korunması, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN payload bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1337/1337** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1337 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d84c6e390b3e5f2b889fc1898cfe68d63d515aaf6b2301fd91c23a404f1948d3`, birleşik referans `f01f56d2abc01b2e6a0ba297a21b7dd3142493d30df1c618e38bf0a52a708975` kimliğindedir. Math karşılaştırma raporları `compare-7zvwnrbm` (debug) ve `compare-on7ckea` (release); birleşik raporlar `compare-4l5i3tbl` (debug) ve `compare-f9mdmv2u` (release), Java doğrulaması `verify-zae5dspu` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.with(Direction.Axis, double)` sonucunun ham f64 snapshot bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
