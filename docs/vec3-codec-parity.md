# P01.b — Vec3 CODEC doğrulaması

`src/math.rs`, Java `Vec3.CODEC` sabitinin sabit üç öğeli liste sözleşmesini `Vec3::codec_decode` ve `Vec3::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı raw f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Vec3 CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam üç öğeli decode x/y/z sırasını ve signed zero, subnormal, Infinity ve NaN bitlerini korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk üç öğeden partial Vec3 üretti.
- Rust debug ve release adaptörleri güncel 1092 senaryoluk math ailesinde **1092/1092** eşleşti.
- Birleşik altı aile corpus'u **1594/1594** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1594 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `8475d3304542000c5493cb030a2dfb280baa6a64c19a4ba3eb09d75f9c0e67a8`. Birleşik referans kimliği: `a15082ba584c6d0bca4bc52d80fd5da8fc49f05193cec75afd71cf0af0c4ee0d`.

Yerel raporlar: `compare-ta5w_83f` (math debug), `compare-vzkk24q9` (math release), `compare-271dl4fv` (birleşik debug), `compare-y8i46uni` (birleşik release) ve `verify-3t751q44` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `Vec3.CODEC` içindeki üç öğeli sabit boyut dönüşümünü, x/y/z encode sırasını, raw f64 bitlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, başka codec'lerin bileşimi ve oyun paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
