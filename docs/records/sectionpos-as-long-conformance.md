# P01.c — SectionPos asLong overload'ları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, SectionPos packed long üretiminin BlockPos, koordinat ve instance çağrı yollarıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:215-228` incelendi. Oracle üç Java overload'unu doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`asLong(BlockPos)`, BlockPos'un signed x/y/z bileşenlerini alır, her bileşeni dört bit arithmetic shift ile section koordinatına çevirir ve koordinat overload'una aktarır. `asLong(int, int, int)`, x ve z için 22 bit, y için 20 bit alanı maskeler ve bu alanları SectionPos düğümündeki sabit konumlarına yerleştirir. Instance `asLong()`, mevcut section bileşenlerini aynı koordinat yoluna verir.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Overload yolları | 3 | Origin, section sınırlarını geçen pozitif/negatif BlockPos girdileri |
| Packed alan sınırları | 2 | 22/20 bit pozitif ve negatif section sınırları |
| Keyfi ve i32 girdiler | 1 | Keyfi packed BlockPos ile i32 sınırlarında koordinat/instance maskesi |

Girdiler `family: "math"`, `operation: "sectionpos_as_long"`, onaltılık 16 haneli `block_node` ve signed decimal i32 `x`, `y`, `z`, `section_x`, `section_y`, `section_z` alanlarını kullanır. `result.value` içinde `from_block_pos`, `coordinates` ve `instance` unsigned lowercase hexadecimal packed sonuçları bulunur.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `SectionPos::as_long_from_block_pos`, `as_long_coordinates` ve `as_long`, daha önce doğrulanan BlockPos unpack, section arithmetic shift ve SectionPos field-mask packing yollarını kullanır. `examples/behavior_adapter.rs` sonuçları JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java overload'larını karşılaştırma oracle'ı olarak çağırır.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, SectionPos packed long üretimini packed BlockPos, doğrudan section koordinatları ve instance bileşenleri üzerinden Java ile aynı alan maskeleriyle yapar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos asLong overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos section sınır geçişleri, pozitif/negatif 22/20 bit sınırları, keyfi packed girdiler ve i32 koordinat maskeleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.asLong(BlockPos)`, `asLong(int, int, int)` ve instance `asLong()` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
