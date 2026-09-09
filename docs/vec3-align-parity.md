# P01.b — Vec3 align doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 875 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **875/875** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1329/1329** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1329 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5613b7aa336a4fb4cf729c33b1e4d9566e5c0795c663ab83879f3787157f0831`, birleşik referans `22a5e87b3b46a0afade4d43a8a9e0dfaf96d8d892ea18ddc0ba5ebf77750772c` kimliğindedir. Math karşılaştırma raporları `compare-p4m_t_r8` (debug) ve `compare-eskrxypz` (release); birleşik raporlar `compare-skwbedqw` (debug) ve `compare-9bwuf8cs` (release), Java doğrulaması `verify-5pjzzzfd` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
