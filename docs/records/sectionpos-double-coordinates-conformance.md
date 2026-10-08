# P01.c — SectionPos double koordinat yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `SectionPos.posToSectionCoord(double)` ve `SectionPos.blockToSectionCoord(double)` overload'larının kayan nokta girdisini section koordinatına çevirme yoludur.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:77-87` ve `.data/minecraft/source/net/minecraft/util/Mth.java:66-68` incelendi. Oracle, iki Java overload'unu doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`posToSectionCoord(double)`, girdiye `Mth.floor` uygular ve sonucu int `blockToSectionCoord` yoluna verir. `blockToSectionCoord(double)` da `Mth.floor` sonucunu dört bit arithmetic shift ile section koordinatına çevirir. Negatif kesirler aşağı yuvarlanır. `Mth.floor` içindeki Java `double` → `int` daraltması NaN için sıfır, aralık dışı değerler için signed int sınırlarını korur; ardından section dönüşümünün `>> 4` davranışı uygulanır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| `posToSectionCoord(double)` | 3 | Pozitif sınır, negatif subnormal ve en büyük sonlu double |
| `blockToSectionCoord(double)` | 3 | Pozitif kesir, negatif sınır ve NaN |

Girdiler `family: "math"`, `operation: "sectionpos_double_coordinates"`, `fn` ve 16 haneli unsigned hexadecimal f64 bit desenini taşıyan `value` alanlarını kullanır. `fn`, `pos_to_section` veya `block_to_section` değerlerinden biridir. `result.value`, signed decimal i32 section koordinatıdır.

Bu sözleşme yalnızca iki double overload'unun son int sonucunu kapsar; diğer `Mth.floor` overload'ları ve SectionPos stream/codec yolları ayrı dilimlerdir.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `SectionPos::pos_to_section_coord` ve `block_to_section_coord_f64`, Java `Mth.floor` ile ortak `math::floor_f64` yolunu ve SectionPos'un int section dönüşümünü taşır. `examples/behavior_adapter.rs` f64 bit girdisini bu API'lere bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, SectionPos `posToSectionCoord(double)` ve `blockToSectionCoord(double)` overload'larını Java'nın floor, narrowing cast ve arithmetic shift sonuçlarıyla eşleştirir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos double koordinatları için 6 özgün senaryo hazırlandı.
- İki overload; pozitif/negatif sınırlar, subnormal, en büyük sonlu değer ve NaN girdileriyle doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki SectionPos double overload'unun altı son int çıktısıyla sınırlıdır. `Mth.floor(float)`, `Mth.lfloor(double)`, diğer SectionPos overload'ları ve gerçek istemci kullanım zincirleri ayrı kapsamdır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
