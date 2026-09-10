# P01.c — ChunkPos.CODEC doğrulaması

`src/binary.rs`, Java `ChunkPos.CODEC` sabitinin iki öğeli signed int liste sözleşmesini `ChunkPos::codec_decode` ve `ChunkPos::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı sentetik x/z girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam iki öğeli decode x/z sırasını, negatif değerleri ve i32 sınırlarını korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk iki öğeden partial ChunkPos üretti.
- Rust debug ve release adaptörleri güncel **1450/1450** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1952 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`. Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Yerel raporlar: `compare-qtqxwt9b` (math debug), `compare-ebava42g` (math release), `compare-x0hkkvkv` (birleşik debug), `compare-x5wgwswa` (birleşik release) ve `verify-rc21vyy9` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.CODEC` içindeki iki öğeli sabit boyut dönüşümünü, x/z encode sırasını, signed i32 değerlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, `ChunkPos.STREAM_CODEC` packed long biçimi, region/block yardımcıları ve chunk paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
