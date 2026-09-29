# P01.c — ChunkPos.CODEC sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **12 Java davranış testi hazırlanmıştır**. Kapsam, Java `ChunkPos.CODEC` sabitinin iki signed int'i x ve z koordinatlarına dönüştürmesi ve bir ChunkPos'u aynı sıradaki iki int listesine kodlamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:19-22` içindeki codec tanımı ve `util/Util.java:945-951` içindeki `IntStream` sabit boyut kontrolü incelendi. Oracle, `JsonOps` üzerinde Java `ChunkPos.CODEC` çağrısını doğrudan yapar; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde genel JSON sayı türü ayrıştırması ölçülmez. Sentetik decode girdileri signed decimal int metinlerinden oluşturulan Java `JsonArray` değerleridir. Rust üretim API'si JSON veya `serde_json` almaz; sabit liste sözleşmesini i32 slice ve sabit boyutlu dizi ile taşır.

## Davranış sözleşmesi

- Decode girdisi tam iki int içerirse ilk değer x, ikinci değer z olur ve yeni bir ChunkPos üretilir.
- Decode girdisi iki int'ten kısa veya boşsa `Input is not a list of 2 ints` hatası döner ve partial ChunkPos bulunmaz.
- Decode girdisi ikiden uzunsa aynı hata döner; partial değer ilk iki int'ten oluşturulan ChunkPos'tur. Sonraki değerler partial değere girmez.
- Encode sonucu x ve z koordinatlarını aynı sıradaki iki int listesi olarak taşır.
- i32 sınırları ve negatif değerler Java/Rust karşılaştırmasında signed değerler olarak korunur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Decode tam liste | 5 | Temel, sıfır, negatif/sınır, signed sınır ve alternatif x/z değerleri |
| Decode yanlış boyut | 3 | Boş, kısa ve uzun listeler; hata metni ve uzun listede ilk iki öğenin partial ChunkPos'u |
| Encode | 4 | Temel, sıfır, signed sınır ve alternatif x/z değerleri; iki int'in x/z sırası |

Girdiler `family: "math"`, `operation: "chunkpos_codec"` ve `mode: "decode" | "encode"` kullanır. Decode isteğinde `values` signed decimal int metin listesi; encode isteğinde `x` ve `z` aynı biçimdeki bileşenlerdir. Başarılı decode `result.value` içinde x/z snapshot'ı, başarılı encode iki öğeli int listesi taşır. Hatalı decode `result.outcome`, `result.error` ve uzun liste için `result.partial` alanlarını taşır.

## Rust karşılığı

`crates/voxel-core/src/binary.rs` içindeki `ChunkPos::codec_decode`, tam iki öğeyi kabul eder; yanlış boyutta gerçek liste uzunluğunu ve yalnızca iki veya daha fazla öğe için ilk iki öğeden oluşan partial değeri `ChunkPosCodecError::WrongLength` içinde taşır. `ChunkPos::codec_encode`, bileşenleri `[x, z]` sırasıyla döndürür. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java codec sonucunu doğrudan `DataResult` üzerinden gözlemler.


## Client-parity doğrulaması

`crates/voxel-core/src/binary.rs`, Java `ChunkPos.CODEC` sabitinin iki öğeli signed int liste sözleşmesini `ChunkPos::codec_decode` ve `ChunkPos::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı sentetik x/z girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam iki öğeli decode x/z sırasını, negatif değerleri ve i32 sınırlarını korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk iki öğeden partial ChunkPos üretti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.CODEC` içindeki iki öğeli sabit boyut dönüşümünü, x/z encode sırasını, signed i32 değerlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, `ChunkPos.STREAM_CODEC` packed long biçimi, region/block yardımcıları ve chunk paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
