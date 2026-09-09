# P01.b — Vec3 with doğrulaması

`src/math.rs`, Java `Vec3.with(Direction.Axis, double)` metodunu karşılar. Rust adaptörü seçilen bileşeni verilen f64 değere taşır ve diğer bileşen bitlerini korur; Java oracle doğrudan `Vec3.with` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 941 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **941/941** math senaryosunda eşleşti.
- Beş yeni senaryoda X/Y/Z seçimi, değiştirilmemiş bileşenlerin korunması, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN payload bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1443/1443** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1443 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7aaf16bdfc85f793bd25e0d4d234f6988f30874d554dc65c32ce3d0ec4f85363`, birleşik referans `998945022ecb5aa3b303b59cbf3789de5cd227359a784553dadb87da8ba02eb6` kimliğindedir. Math karşılaştırma raporları `compare-9rd6f2k4` (debug) ve `compare-azcslb8q` (release); birleşik raporlar `compare-9epmwbb2` (debug) ve `compare-vdfamuam` (release), Java doğrulaması `verify-povrla2w` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.with(Direction.Axis, double)` sonucunun ham f64 snapshot bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
