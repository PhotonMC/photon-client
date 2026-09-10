# P01.b — Vec3 CODEC doğrulaması

`src/math.rs`, Java `Vec3.CODEC` sabitinin sabit üç öğeli liste sözleşmesini `Vec3::codec_decode` ve `Vec3::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı raw f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Vec3 CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam üç öğeli decode x/y/z sırasını ve signed zero, subnormal, Infinity ve NaN bitlerini korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk üç öğeden partial Vec3 üretti.
- Rust debug ve release adaptörleri güncel 1608 senaryoluk math ailesinde **1608/1608** eşleşti.
- Birleşik altı aile corpus'u **2110/2110** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2110 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `10c179a43201ead1d005dbd3e8f1deb7d1b4d5928bd228214792c84e07ef2b41`. Birleşik referans kimliği: `bb3698ca7b34e45053dd8865ed3b754080b1fb665c802937b38ccb0216ecede3`.

Yerel raporlar: `compare-pmcommj0` (math debug), `compare-4g5l34ea` (math release), `compare-mki3lw11` (birleşik debug), `compare-r66knife` (birleşik release) ve `verify-35w0iz71` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `Vec3.CODEC` içindeki üç öğeli sabit boyut dönüşümünü, x/y/z encode sırasını, raw f64 bitlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, başka codec'lerin bileşimi ve oyun paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
