# P01.b — Vec3 align doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1963 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1963/1963** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2465 senaryoluk Java referansını yeniden doğruladı.

Math referansı `461f1ba035270ce58c69f06fec7167bb0517cd22aed6c78c25e0b076684b28fa`, birleşik referans `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573` kimliğindedir. Math karşılaştırma raporları `compare-hiau72j0` (debug) ve `compare-bttpme0o` (release); birleşik raporlar `compare-k17hah5m` (debug) ve `compare-hiiz6u35` (release), Java doğrulaması `verify-82qi_x56` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
