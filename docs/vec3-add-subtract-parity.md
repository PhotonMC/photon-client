# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1963 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1963/1963** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2465 senaryoluk Java referansını yeniden doğruladı.

Math referansı `461f1ba035270ce58c69f06fec7167bb0517cd22aed6c78c25e0b076684b28fa`, birleşik referans `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573` kimliğindedir.

Yerel raporlar: `compare-hiau72j0` (math debug), `compare-bttpme0o` (math release), `compare-k17hah5m` (birleşik debug), `compare-hiiz6u35` (birleşik release) ve `verify-82qi_x56` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
