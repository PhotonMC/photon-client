# P01.d — AABB deflate üç eksenli overload doğrulaması

`src/aabb.rs`, Java `AABB.deflate(double, double, double)` overload'unun ters işaretli `inflate` davranışını `Aabb::deflate_xyz` ile uygular. Java oracle üç miktarlı overload'u doğrudan çağırır; Rust adapter aynı girdiyi üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 660 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **660/660** math senaryosunda eşleşti.
- Beş yeni senaryoda karışık eksen miktarları, constructor normalizasyonu, signed zero, zincir ve NaN/sonsuzluk yolları karşılaştırıldı.
- NaN senaryosunda release derlemesinin Java unary negation bitlerini koruduğu ayrıca doğrulandı.
- Birleşik altı aile corpus'u **1114/1114** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1114 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bc87ac5ece4a47cf40d4e562d69ac72206437e2c117cd058a79561ba4b4d149b`, birleşik referans `15dc52b0af0addc9b446d43666846404d2d8e35a7cc7751489632ec703b826b8` kimliğindedir.

Yerel raporlar: `compare-3kg8m9ta` (math debug), `compare-6crwfl2x` (math release), `compare-_35t8r3l` (birleşik debug), `compare-g5oda9d_` (birleşik release) ve `verify-6xepcluv` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik AABB step dizisi ve bunların endpoint snapshot'larıyla sınırlıdır. Üç eksenli overload'un doğrudan davranışı doğrulandı; istemci hareketi ve dünya collider toplama sonraki P01/P09 dilimleridir.
