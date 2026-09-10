# P01.c — SectionPos double koordinat doğrulaması

`src/binary.rs`, SectionPos `posToSectionCoord(double)` ve `blockToSectionCoord(double)` overload'larını Java'nın floor, narrowing cast ve arithmetic shift sonuçlarıyla eşleştirir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos double koordinatları için 6 özgün senaryo hazırlandı.
- İki overload; pozitif/negatif sınırlar, subnormal, en büyük sonlu değer ve NaN girdileriyle doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1763/1763** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2265/2265** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `7e0ba63794c7ec98b0e5be4494e1fd999edd354071389371ea9497d0096242aa`. Birleşik referans kimliği: `0ad19435d5e4101420352fbb4920646f636d51c748f4cfe43f9a29d2da727cd5`.

Adapter hashleri: debug `0d3406b9f649da89947fcf975aa7e45601195191d4845847803216ab69b7b923`, release `7f8d8d0d89a26ed30abf37dc651545a2304ed76be50bbede380384e51e304d98`.

Yerel raporlar: `compare-7mgwi148` (math debug), `compare-rq3_yvog` (math release), `compare-efi7e5qh` (birleşik debug), `compare-qw_kmael` (birleşik release), `verify-tljqi7wo` (math) ve `verify-oxhrmmem` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki SectionPos double overload'unun altı son int çıktısıyla sınırlıdır. `Mth.floor(float)`, `Mth.lfloor(double)`, diğer SectionPos overload'ları ve gerçek istemci kullanım zincirleri ayrı kapsamdır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
