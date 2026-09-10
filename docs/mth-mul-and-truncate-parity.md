# P01.a — Mth mulAndTruncate doğrulaması

`src/math.rs`, Java `Mth.mulAndTruncate(Fraction, int)` işlemini Fraction'ın numerator/denominator alanları üzerinden i32 çarpma ve sıfıra doğru bölüm sırasıyla taşır. `examples/behavior_adapter.rs` sonucu signed decimal i32 olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Pozitif ve negatif kesirleri, sıfır/negatif factor değerlerini, indirgenmemiş Fraction alanlarını, numerator/denominator sınırlarını ve i32 çarpma taşmalarını kapsayan toplam **20 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1450/1450** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`. Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Adapter hashleri: debug `1a871a434c144a587a2723327bd46aaf9a6d1b4612396a66fffe175369c31d12`, release `8c7871587c66a581ea4b33d57a1ef23281843488a7661885a1a704dee0ad3e74`.

Yerel raporlar: `compare-qtqxwt9b` ve `compare-ebava42g` (math debug/release), `compare-x0hkkvkv` ve `compare-x5wgwswa` (birleşik debug/release), `verify-m2eek8to` (math) ve `verify-rc21vyy9` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.mulAndTruncate` helper'ının pozitif ve sıfır olmayan denominator ile oluşturulan sabit Fraction alanları, Java i32 taşması ve truncating bölüm sonuçları içindir. Fraction factory hata yolları, diğer Fraction işlemleri ve bu helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
