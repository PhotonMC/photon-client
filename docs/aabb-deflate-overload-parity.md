# P01.d — AABB deflate üç eksenli overload doğrulaması

`src/aabb.rs`, Java `AABB.deflate(double, double, double)` overload'unun ters işaretli `inflate` davranışını `Aabb::deflate_xyz` ile uygular. Java oracle üç miktarlı overload'u doğrudan çağırır; Rust adapter aynı girdiyi üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1378 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1378/1378** math senaryosunda eşleşti.
- Beş yeni senaryoda karışık eksen miktarları, constructor normalizasyonu, signed zero, zincir ve NaN/sonsuzluk yolları karşılaştırıldı.
- NaN senaryosunda release derlemesinin Java unary negation bitlerini koruduğu ayrıca doğrulandı.
- Birleşik altı aile corpus'u **1880/1880** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1880 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6d35da84e1eb81c28088bf3f4951651b252daed29eda65c96f035389b897e6e4`, birleşik referans `d1375ef4dcd71b8eb3c95f84ca200ef0ed3303aa7500447f6683cb24a4bd1d81` kimliğindedir.

Yerel raporlar: `compare-_ohp5wb8` (math debug), `compare-bezugwpw` (math release), `compare-i4o7vofo` (birleşik debug), `compare-9vhvfokc` (birleşik release) ve `verify-kfxghdw3` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik AABB step dizisi ve bunların endpoint snapshot'larıyla sınırlıdır. Üç eksenli overload'un doğrudan davranışı doğrulandı; istemci hareketi ve dünya collider toplama sonraki P01/P09 dilimleridir.
