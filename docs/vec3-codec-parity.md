# P01.b — Vec3 CODEC doğrulaması

`src/math.rs`, Java `Vec3.CODEC` sabitinin sabit üç öğeli liste sözleşmesini `Vec3::codec_decode` ve `Vec3::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı raw f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Vec3 CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam üç öğeli decode x/y/z sırasını ve signed zero, subnormal, Infinity ve NaN bitlerini korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk üç öğeden partial Vec3 üretti.
- Rust debug ve release adaptörleri güncel 1025 senaryoluk math ailesinde **1025/1025** eşleşti.
- Birleşik altı aile corpus'u **1527/1527** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1527 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `2c79e2aaba1154782177df83cd39488c1c9a365c76e1971733e837c447fe9fa3`. Birleşik referans kimliği: `74f88dfa13fb755a92b838953181016d177ef3a05b113b5fa8ef1f3735031acc`.

Yerel raporlar: `compare-0dsoh6yn` (math debug), `compare-wdh5yokb` (math release), `compare-7wace8y9` (birleşik debug), `compare-ohni39xi` (birleşik release) ve `verify-a7iqdyvs` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `Vec3.CODEC` içindeki üç öğeli sabit boyut dönüşümünü, x/y/z encode sırasını, raw f64 bitlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, başka codec'lerin bileşimi ve oyun paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
