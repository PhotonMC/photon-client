# P01.b — Vec3 align doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1221 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1221/1221** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1723/1723** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1723 senaryoluk Java referansını yeniden doğruladı.

Math referansı `df8b0fd452c7f12175fe9ced3c534c1f64bdb7df75cc28529becd0eb8ab97b3c`, birleşik referans `86380f621c6f188dddfa1e68a94057a8ada6d638b9f5c04d3a2c9682b1ca75cf` kimliğindedir. Math karşılaştırma raporları `compare-6dvmyv_h` (debug) ve `compare-ak0n9_u3` (release); birleşik raporlar `compare-_8esk0jy` (debug) ve `compare-g98k9sn0` (release), Java doğrulaması `verify-72fv37y0` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
