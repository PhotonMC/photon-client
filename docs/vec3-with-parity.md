# P01.b — Vec3 with doğrulaması

`src/math.rs`, Java `Vec3.with(Direction.Axis, double)` metodunu karşılar. Rust adaptörü seçilen bileşeni verilen f64 değere taşır ve diğer bileşen bitlerini korur; Java oracle doğrudan `Vec3.with` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1092 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1092/1092** math senaryosunda eşleşti.
- Beş yeni senaryoda X/Y/Z seçimi, değiştirilmemiş bileşenlerin korunması, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN payload bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1594/1594** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1594 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8475d3304542000c5493cb030a2dfb280baa6a64c19a4ba3eb09d75f9c0e67a8`, birleşik referans `a15082ba584c6d0bca4bc52d80fd5da8fc49f05193cec75afd71cf0af0c4ee0d` kimliğindedir. Math karşılaştırma raporları `compare-ta5w_83f` (debug) ve `compare-vzkk24q9` (release); birleşik raporlar `compare-271dl4fv` (debug) ve `compare-y8i46uni` (release), Java doğrulaması `verify-3t751q44` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.with(Direction.Axis, double)` sonucunun ham f64 snapshot bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
