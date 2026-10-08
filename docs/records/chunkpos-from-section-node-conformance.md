# P01.c — ChunkPos section node dönüşümü sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed SectionPos long değerinden ChunkPos packed long üretme yoludur.

## Kaynak ve giriş biçimi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:77-79` ve `core/SectionPos.java:136-145` yolları incelendi. Oracle, `ChunkPos.fromSectionNode(long)` metodunu doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Her `section_node` girdisi, SectionPos packed long’unun ham bitlerini taşıyan 16 haneli küçük harfli unsigned hex metnidir. SectionPos x/z alanları 22 bit signed değerlerdir; y alanı 20 bittir.

## Davranış sözleşmesi

- Java, section node içinden SectionPos x ve z alanlarını signed 22 bit olarak çıkarır.
- Section y alanı dönüşümde kullanılmaz.
- Çıkarılan x düşük, z yüksek 32 bit alanlara yerleştirilerek ChunkPos packed long’u üretilir.
- Section x/z uçlarındaki işaret genişletme ve ChunkPos packed long’un unsigned hex gösterimi korunur.

Bu dilimde SectionPos üreticileri, section node offset işlemleri, BlockPos dönüşümleri ve `ChunkPos.rangeClosed` ölçülmez.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| ChunkPos from section node | 6 | Köken, pozitif koordinatlar, tümü negatif koordinatlar, signed 22 bit minimum/maksimum uçları ve y alanının atıldığı packed node |

Girdiler `family: "math"`, `operation: "chunkpos_from_section_node"` ve `section_node` alanını kullanır. `result.value`, Java long sonucunun unsigned lowercase hex metnidir.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `ChunkPos::from_section_node`, mevcut `SectionPos` packed long çözümünden x/z değerlerini alıp ChunkPos `pack` yoluna verir. `examples/behavior_adapter.rs` sonucu JSONL’ye bağlar; `tests/behavior/java/MathOracle.java` Java metodunu doğrudan gözlemler.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, Java `ChunkPos.fromSectionNode(long)` yolunu SectionPos packed long’undan signed x/z alanlarını çıkarıp ChunkPos packed long’una dönüştürerek karşılar. Rust adaptörü sonucu Java ile aynı unsigned lowercase hex biçiminde taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos section node dönüşümü için 6 özgün senaryo hazırlandı.
- Pozitif/negatif section koordinatları, signed 22 bit minimum/maksimum uçları ve y alanının atılması eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.fromSectionNode(long)` dönüşümünün x/z çıkarma, y atma ve packed long yerleşimini kapsar. SectionPos üreticileri, node offset işlemleri, BlockPos dönüşümleri, `ChunkPos.rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
