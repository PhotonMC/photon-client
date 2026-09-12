# P01.c — SectionPos offset overload'ları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed SectionPos düğümünü Direction veya üç i32 adımla kaydıran static yollar ile SectionPos instance kaydırma yoludur.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:69-75,230-232` incelendi. Oracle static Direction, static üç adım ve instance `offset` yollarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`offset(long, Direction)`, yönün x/y/z step değerlerini üç adımlı static overload'a verir. Bu yol packed SectionPos alanlarını signed olarak açar, adımları Java i32 toplamasıyla ekler ve sonucu `asLong` alan maskeleriyle yeniden pack eder. Instance `offset(int, int, int)`, üç adımın tamamı sıfırsa mevcut SectionPos'u korur; aksi halde bileşenleri aynı Java i32 wrapping toplamasıyla yeni SectionPos'a taşır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Overload yolları | 3 | Sıfır, pozitif ve negatif packed/instance kaydırmalar; Direction adımları |
| Packed alan sınırı | 1 | Pozitif x/y/z alan sınırında maskelenen static sonuç |
| i32 taşması | 1 | Packed static toplama ve instance bileşen taşması |
| Keyfi zero yolu | 1 | Keyfi packed node ve sıfır instance adımı |

Girdiler `family: "math"`, `operation: "sectionpos_offset"`, 16 haneli unsigned hex `section_node`, küçük harfli `direction`, signed decimal i32 `step_x`, `step_y`, `step_z`, `x`, `y`, `z` alanlarını kullanır. `result.value` içinde Direction static yolu `packed_direction`, üç adımlı static yol `packed_steps`, instance bileşenleri ve instance packed sonucu bulunur.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::offset_packed`, `offset_packed_by` ve `offset`, daha önce doğrulanan Direction step, signed SectionPos unpack/pack ve i32 wrapping yollarını kullanır. `examples/behavior_adapter.rs` sonuçları JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java overload'larını karşılaştırma oracle'ı olarak çağırır.


## Client-parity doğrulaması

`src/binary.rs`, SectionPos packed düğümlerini Direction veya üç adımla kaydırır ve instance koordinatlarını Java ile aynı i32 toplama ve packed field-mask yollarıyla üretir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos offset overload'ları için 6 özgün senaryo hazırlandı.
- Direction/static/instance çağrı yolları, sıfır adım, pozitif/negatif değerler, packed alan sınırı ve i32 taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.offset(long, Direction)`, `offset(long, int, int, int)` ve instance `offset(int, int, int)` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
