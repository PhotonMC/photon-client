# P01.c — ChunkPos.CODEC doğrulaması

`src/binary.rs`, Java `ChunkPos.CODEC` sabitinin iki öğeli signed int liste sözleşmesini `ChunkPos::codec_decode` ve `ChunkPos::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı sentetik x/z girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam iki öğeli decode x/z sırasını, negatif değerleri ve i32 sınırlarını korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk iki öğeden partial ChunkPos üretti.
- Rust debug ve release adaptörleri güncel **2270/2270** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2772/2772** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2772 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.CODEC` içindeki iki öğeli sabit boyut dönüşümünü, x/z encode sırasını, signed i32 değerlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, `ChunkPos.STREAM_CODEC` packed long biçimi, region/block yardımcıları ve chunk paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
