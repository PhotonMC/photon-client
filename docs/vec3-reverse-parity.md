# P01.b — Vec3 reverse doğrulaması

`src/math.rs`, Java `Vec3.reverse()` metodunun `scale(-1.0)` yönlendirmesini karşılar. Java oracle doğrudan çağrıyı yapar; Rust adaptörü aynı girdiyi `Vec3::reverse` API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 959 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **959/959** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu bileşenler, ardışık işlem, signed zero, subnormal, büyük sonlu ve Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1461/1461** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1461 senaryoluk Java referansını yeniden doğruladı.

Math referansı `35103d673698aa0dcef8706322bcfccb0a54e6545c75f67308f6f27f498c922d`, birleşik referans `2fb61f9fd2ce23842225b3592a92ff5a162ad0567561f4606996f2ddbc58842d` kimliğindedir.

Yerel raporlar: `compare-cuylkpl7` (math debug), `compare-iuha2872` (math release), `compare-o2800g_u` (birleşik debug), `compare-7ryigcxt` (birleşik release) ve `verify-jnzrnmg2` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve başlangıç/her adım snapshot'larıyla sınırlıdır. `scale`, `horizontal`, `offsetRandom`, `horizontalDistance`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
