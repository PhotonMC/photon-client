# P01.b — Vec3 with doğrulaması

`src/math.rs`, Java `Vec3.with(Direction.Axis, double)` metodunu karşılar. Rust adaptörü seçilen bileşeni verilen f64 değere taşır ve diğer bileşen bitlerini korur; Java oracle doğrudan `Vec3.with` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1999 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1999/1999** math senaryosunda eşleşti.
- Beş yeni senaryoda X/Y/Z seçimi, değiştirilmemiş bileşenlerin korunması, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN payload bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2501/2501** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2501 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d0cf0bb203c8ea48ad8144fae6a58022846d1a5cfb95b398018620bae7b0bc3d`, birleşik referans `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040` kimliğindedir. Math karşılaştırma raporları `compare-druvbg3h` (debug) ve `compare-k2ed8y70` (release); birleşik raporlar `compare-h1kbcaxz` (debug) ve `compare-js7_mas4` (release), Java doğrulaması `verify-d2ptfzig` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.with(Direction.Axis, double)` sonucunun ham f64 snapshot bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
