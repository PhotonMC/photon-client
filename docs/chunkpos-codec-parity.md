# P01.c — ChunkPos.CODEC doğrulaması

`src/binary.rs`, Java `ChunkPos.CODEC` sabitinin iki öğeli signed int liste sözleşmesini `ChunkPos::codec_decode` ve `ChunkPos::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı sentetik x/z girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam iki öğeli decode x/z sırasını, negatif değerleri ve i32 sınırlarını korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk iki öğeden partial ChunkPos üretti.
- Rust debug ve release adaptörleri güncel **1743/1743** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2245/2245** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2245 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `6b06e5ddd2219dc868e358d02f9afcfbda6469cddd7391f150f9280440124f6a`. Birleşik referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Yerel raporlar: `compare-q3ryrs_k` (math debug), `compare-xtfhknri` (math release), `compare-ag5oxl30` (birleşik debug), `compare-fkrk2srf` (birleşik release) ve `verify-lstfjdye` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.CODEC` içindeki iki öğeli sabit boyut dönüşümünü, x/z encode sırasını, signed i32 değerlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, `ChunkPos.STREAM_CODEC` packed long biçimi, region/block yardımcıları ve chunk paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
