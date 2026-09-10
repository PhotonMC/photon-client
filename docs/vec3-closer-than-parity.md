# P01.b — Vec3 closerThan doğrulaması

`src/math.rs`, Java `Vec3.closerThan(Position, double)` ve `Vec3.closerThan(Vec3, double, double)` overload'larının strict eşiklerini, x/z toplama sırasını ve y koşulunu karşılar. Java oracle iki overload'u doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1638 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1638/1638** math senaryosunda eşleşti.
- Beş yeni senaryoda iç eşikler, strict sınır eşitliği, ayrı y kapısı, negatif sıfır eşikleri ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2140 senaryoluk Java referansını yeniden doğruladı.

Math referansı `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`, birleşik referans `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f` kimliğindedir.

Yerel raporlar: `compare-ep8fiu6` (math debug), `compare-z9iwcisf` (math release), `compare-a21c2t4m` (birleşik debug), `compare-uwg2_ewv` (birleşik release) ve `verify-yscj0_fq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki closerThan overload'unun boolean sonuçlarıyla sınırlıdır. `distanceToSqr` ve Mth yardımcıları ayrı dilimlerdir; Position arayüzünün Vec3 dışındaki uygulamaları bu adaptörde temsil edilmez.
