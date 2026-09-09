# P01.c — SectionPos double koordinat doğrulaması

`src/binary.rs`, SectionPos `posToSectionCoord(double)` ve `blockToSectionCoord(double)` overload'larını Java'nın floor, narrowing cast ve arithmetic shift sonuçlarıyla eşleştirir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos double koordinatları için 6 özgün senaryo hazırlandı.
- İki overload; pozitif/negatif sınırlar, subnormal, en büyük sonlu değer ve NaN girdileriyle doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1149/1149** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1651/1651** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `2f973f986814298b094b982753eef5a170865c5228762b233a6060766b5ed32e`. Birleşik referans kimliği: `f8cfd3cf28e07e5859a339a4844b50e474cb8351ea003bb36ba96ccaf38abec1`.

Adapter hashleri: debug `4ba8fcc5baa3e50779e4ce239b18341b3f611b9af0a363f60520810a3220ba6a`, release `b53b10fe28e9d1ebb14986a6150d650c247091bfab362e793b57a2a4b6e068c9`.

Yerel raporlar: `compare-hz521aor` (math debug), `compare-uded0zec` (math release), `compare-nc1496mg` (birleşik debug), `compare-ljx3yrp0` (birleşik release), `verify-xa5laefb` (math) ve `verify-k950tc5l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki SectionPos double overload'unun altı son int çıktısıyla sınırlıdır. `Mth.floor(float)`, `Mth.lfloor(double)`, diğer SectionPos overload'ları ve gerçek istemci kullanım zincirleri ayrı kapsamdır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
