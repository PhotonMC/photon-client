# P01.c — SectionPos blok koordinatı yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir SectionPos section koordinatının her eksendeki minimum ve maksimum blok koordinatlarına çevrilmesidir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:128-134,160-180` incelendi. Oracle `SectionPos.of(int, int, int)` ile SectionPos oluşturur ve `minBlockX/Y/Z` ile `maxBlockX/Y/Z` instance metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Minimum blok koordinatı, ilgili section koordinatının Java i32 dört bit sola kaydırılmasıdır. Maksimum koordinat, aynı minimum değere 15 eklenerek üretilir. x, y ve z eksenleri bağımsızdır; shift ve toplama i32 sarma davranışı korunur.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Blok koordinatı sınırları | 6 | Köken, pozitif/negatif section'lar, shift sınırı ve i32 koordinat sınırları |

Girdiler `family: "math"`, `operation: "sectionpos_block_coordinates"` ve signed decimal i32 `x`, `y`, `z` alanlarını kullanır. `result.value` içinde `min_block_x/y/z` ve `max_block_x/y/z` signed decimal string sonuçları bulunur.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `SectionPos::min_block_x`, `min_block_y`, `min_block_z`, `max_block_x`, `max_block_y` ve `max_block_z`, ortak section-to-block dönüşümünü ve Java i32 wrapping `+15` davranışını taşır. `examples/behavior_adapter.rs` SectionPos girdisini bu API'lere bağlar; `tests/behavior/java/MathOracle.java` Java instance yardımcılarını karşılaştırma oracle'ı olarak çağırır.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, SectionPos x/y/z section koordinatlarını Java `minBlockX/Y/Z` ve `maxBlockX/Y/Z` sonuçlarıyla aynı blok sınırlarına çevirir. Shift ve maksimum uçtaki `+15` i32 sarma davranışı korunur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `7ca32df88b2b9428c0f70c0e484bc8528524a081223d5abc6f31856bb87963f8`, release `0b8de97e52cf25fab7836202ea3b5a09d7db759ce6fab368fea489722e507f79`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos section koordinatlarının altı min/max blok sınırı helper'ına çevrilmesiyle sınırlıdır. SectionPos üreticileri, packed erişimcileri, relative konumlar, origin/center/chunk dönüşümleri ve stream codec tüketimi ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
