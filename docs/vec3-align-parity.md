# P01.b — Vec3 align doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1903 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1903/1903** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2405/2405** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2405 senaryoluk Java referansını yeniden doğruladı.

Math referansı `0cc821a10c9af88c8563c153e55d57b691a8c69e33d90a8bd3022a5b3410bfd0`, birleşik referans `2daaa6bb28eca57adfb18563e32f011d764209b98ec6de5934a92a72e80abdfa` kimliğindedir. Math karşılaştırma raporları `compare-xiw125c3` (debug) ve `compare-v5h63z7q` (release); birleşik raporlar `compare-chdvlz2_` (debug) ve `compare-oywxz0ky` (release), Java doğrulaması `verify-da0drjib` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
