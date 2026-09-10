# P01.b — Vec3 get doğrulaması

`src/math.rs`, Java `Vec3.get(Direction.Axis)` metodunu karşılar. Rust adaptörü seçilen eksenin bileşen bitlerini döndürür; Java oracle doğrudan `Vec3.get` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1653 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1653/1653** math senaryosunda eşleşti.
- Beş yeni senaryoda üç eksen, signed zero, subnormal, en büyük sonlu değer, Infinity ve payload taşıyan NaN ham bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2155 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`, birleşik referans `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e` kimliğindedir. Math karşılaştırma raporları `compare-ceusm4y4` (debug) ve `compare-hconpy1a` (release); birleşik raporlar `compare-rhjx0z7e` (debug) ve `compare-jtl411eu` (release), Java doğrulaması `verify-u5d5zd4c` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.get(Direction.Axis)` sonucunun ham f64 bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
