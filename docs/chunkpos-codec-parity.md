# P01.c — ChunkPos.CODEC doğrulaması

`src/binary.rs`, Java `ChunkPos.CODEC` sabitinin iki öğeli signed int liste sözleşmesini `ChunkPos::codec_decode` ve `ChunkPos::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı sentetik x/z girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam iki öğeli decode x/z sırasını, negatif değerleri ve i32 sınırlarını korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk iki öğeden partial ChunkPos üretti.
- Rust debug ve release adaptörleri güncel **1506/1506** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2008 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `521c517003287d78e7dee051e04062b28c1fdcf7965213cb691cb7e1e0a3f9df`. Birleşik referans kimliği: `8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57`.

Yerel raporlar: `compare-wn40y3iw` (math debug), `compare-9phkvs2k` (math release), `compare-a415_00_` (birleşik debug), `compare-djb85w4w` (birleşik release) ve `verify-nqxvpkjm` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.CODEC` içindeki iki öğeli sabit boyut dönüşümünü, x/z encode sırasını, signed i32 değerlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, `ChunkPos.STREAM_CODEC` packed long biçimi, region/block yardımcıları ve chunk paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
