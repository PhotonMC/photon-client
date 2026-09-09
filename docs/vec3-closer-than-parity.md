# P01.b — Vec3 closerThan doğrulaması

`src/math.rs`, Java `Vec3.closerThan(Position, double)` ve `Vec3.closerThan(Vec3, double, double)` overload'larının strict eşiklerini, x/z toplama sırasını ve y koşulunu karşılar. Java oracle iki overload'u doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 905 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **905/905** math senaryosunda eşleşti.
- Beş yeni senaryoda iç eşikler, strict sınır eşitliği, ayrı y kapısı, negatif sıfır eşikleri ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1407/1407** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1407 senaryoluk Java referansını yeniden doğruladı.

Math referansı `847816d7bf7d2e0e9cecc67b5d7010685842d00cd4860fb3cf9447d37a9fe8a9`, birleşik referans `a2c69ab98e59ffb82410fd45282bc6e8c110fd13877ba87471c7bfcc29f23318` kimliğindedir.

Yerel raporlar: `compare-psz_s0y8` (math debug), `compare-xhpjkx1_` (math release), `compare-smgo22vh` (birleşik debug), `compare-h2kr9uj0` (birleşik release) ve `verify-fki2znkz` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki closerThan overload'unun boolean sonuçlarıyla sınırlıdır. `distanceToSqr` ve Mth yardımcıları ayrı dilimlerdir; Position arayüzünün Vec3 dışındaki uygulamaları bu adaptörde temsil edilmez.
