# P01.c — SectionPos packed koordinat erişimcileri sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed SectionPos long içinden signed x, y ve z section koordinatlarının çıkarılmasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:136-145` incelendi. Oracle `SectionPos.x(long)`, `SectionPos.y(long)` ve `SectionPos.z(long)` metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Packed giriş, 64 bit ham değeri temsil eden 16 haneli küçük harfli unsigned hex metnidir. x, bit 42–63 arasındaki signed 22 bit alanından; z, bit 20–41 arasındaki signed 22 bit alanından; y ise bit 0–19 arasındaki signed 20 bit alanından okunur. Her alan Java arithmetic shift sign extension davranışıyla signed i32 değerine taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Packed alan erişimi | 6 | Sıfır, pozitif ve negatif alanlar, tüm alanların alt/üst sınır desenleri ve alternatif bit deseni |

Girdiler `family: "math"`, `operation: "sectionpos_packed_accessors"` ve 16 haneli `packed` alanını kullanır. `result.value.x`, `result.value.y` ve `result.value.z`, signed decimal string sonuçlarıdır.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `SectionPos::get_x`, `get_y` ve `get_z`, packed değerin 22/20/22 bit alanlarını Java ile aynı sign extension işlemiyle döndürür. `SectionPos::unpack` bu erişimcileri kullanır. `examples/behavior_adapter.rs` ham packed hex girdisini bu API'lere bağlar; `tests/behavior/java/MathOracle.java` karşılaştırma oracle'ı olarak Java erişimcilerini çağırır.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, packed SectionPos long içindeki signed x, y ve z alanlarını Java `SectionPos.x(long)`, `y(long)` ve `z(long)` ile aynı i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API'lere bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, signed 22/20 bit uçları ve alternatif bit deseni doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `f8fafbf2d2cd3ad40cd739a394f6d29f2575d4a9a3d7e85b6266879b4ce57682`, release `30554a139fc26f799697f0ab5c6ef3f2668d976b96dfa59d85ba7f2f9d7d6e96`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından section x/y/z çıkarma ve signed sign extension yorumuyla sınırlıdır. SectionPos üreticileri, blok dönüşümleri, relative konumlar, stream codec tüketimi ve bu accessor'ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
