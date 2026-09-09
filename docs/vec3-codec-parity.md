# P01.b — Vec3 CODEC doğrulaması

`src/math.rs`, Java `Vec3.CODEC` sabitinin sabit üç öğeli liste sözleşmesini `Vec3::codec_decode` ve `Vec3::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı raw f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Vec3 CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam üç öğeli decode x/y/z sırasını ve signed zero, subnormal, Infinity ve NaN bitlerini korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk üç öğeden partial Vec3 üretti.
- Rust debug ve release adaptörleri güncel 965 senaryoluk math ailesinde **965/965** eşleşti.
- Birleşik altı aile corpus'u **1467/1467** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1467 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `bbc2c1dc1c594cc962e6acbf6fcdc98815c896f66dcad6a72c2da4be5b0a0f81`. Birleşik referans kimliği: `5cb121b3bc1e12571ad171166a785300fd6e581cf78813e4ddaf2b99d19ab7d1`.

Yerel raporlar: `compare-ctepk6ji` (math debug), `compare-5wubt_if` (math release), `compare-lsdsnr89` (birleşik debug), `compare-b5y_ka7p` (birleşik release) ve `verify-o3reu3wd` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `Vec3.CODEC` içindeki üç öğeli sabit boyut dönüşümünü, x/y/z encode sırasını, raw f64 bitlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, başka codec'lerin bileşimi ve oyun paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
