# P01.b — Vec3 closerThan doğrulaması

`src/math.rs`, Java `Vec3.closerThan(Position, double)` ve `Vec3.closerThan(Vec3, double, double)` overload'larının strict eşiklerini, x/z toplama sırasını ve y koşulunu karşılar. Java oracle iki overload'u doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 923 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **923/923** math senaryosunda eşleşti.
- Beş yeni senaryoda iç eşikler, strict sınır eşitliği, ayrı y kapısı, negatif sıfır eşikleri ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1425/1425** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1425 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e11b3ae14a5bdb99014f3283b0ed617e1644a4e15c06be01af7f8b8ef84743d6`, birleşik referans `46678aa48c299841b0c8c187569d7f3358ea4c2098c365f4570a69310eeb7422` kimliğindedir.

Yerel raporlar: `compare-85j8zrrj` (math debug), `compare-eq8g_l1t` (math release), `compare-ufi9ykko` (birleşik debug), `compare-fuqao9g1` (birleşik release) ve `verify-sqpfj18l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki closerThan overload'unun boolean sonuçlarıyla sınırlıdır. `distanceToSqr` ve Mth yardımcıları ayrı dilimlerdir; Position arayüzünün Vec3 dışındaki uygulamaları bu adaptörde temsil edilmez.
