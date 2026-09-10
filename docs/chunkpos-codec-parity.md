# P01.c — ChunkPos.CODEC doğrulaması

`src/binary.rs`, Java `ChunkPos.CODEC` sabitinin iki öğeli signed int liste sözleşmesini `ChunkPos::codec_decode` ve `ChunkPos::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı sentetik x/z girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam iki öğeli decode x/z sırasını, negatif değerleri ve i32 sınırlarını korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk iki öğeden partial ChunkPos üretti.
- Rust debug ve release adaptörleri güncel **1534/1534** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2036 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `99f42388f3562f04669e01818a39202ee8f698f0151f153acf5352e96f0fcc58`. Birleşik referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Yerel raporlar: `compare-t6yvipac` (math debug), `compare-6_snmnk3` (math release), `compare-tgx0qoj4` (birleşik debug), `compare-t1bsi_04` (birleşik release) ve `verify-zwp9iban` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.CODEC` içindeki iki öğeli sabit boyut dönüşümünü, x/z encode sırasını, signed i32 değerlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, `ChunkPos.STREAM_CODEC` packed long biçimi, region/block yardımcıları ve chunk paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
