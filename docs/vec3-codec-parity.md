# P01.b — Vec3 CODEC doğrulaması

`src/math.rs`, Java `Vec3.CODEC` sabitinin sabit üç öğeli liste sözleşmesini `Vec3::codec_decode` ve `Vec3::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı raw f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Vec3 CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam üç öğeli decode x/y/z sırasını ve signed zero, subnormal, Infinity ve NaN bitlerini korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk üç öğeden partial Vec3 üretti.
- Rust debug ve release adaptörleri güncel 923 senaryoluk math ailesinde **923/923** eşleşti.
- Birleşik altı aile corpus'u **1425/1425** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1425 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `e11b3ae14a5bdb99014f3283b0ed617e1644a4e15c06be01af7f8b8ef84743d6`. Birleşik referans kimliği: `46678aa48c299841b0c8c187569d7f3358ea4c2098c365f4570a69310eeb7422`.

Yerel raporlar: `compare-85j8zrrj` (math debug), `compare-eq8g_l1t` (math release), `compare-ufi9ykko` (birleşik debug), `compare-fuqao9g1` (birleşik release) ve `verify-sqpfj18l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `Vec3.CODEC` içindeki üç öğeli sabit boyut dönüşümünü, x/y/z encode sırasını, raw f64 bitlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, başka codec'lerin bileşimi ve oyun paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
