# P01.a — Mth mulAndTruncate doğrulaması

`src/math.rs`, Java `Mth.mulAndTruncate(Fraction, int)` işlemini Fraction'ın numerator/denominator alanları üzerinden i32 çarpma ve sıfıra doğru bölüm sırasıyla taşır. `examples/behavior_adapter.rs` sonucu signed decimal i32 olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Pozitif ve negatif kesirleri, sıfır/negatif factor değerlerini, indirgenmemiş Fraction alanlarını, numerator/denominator sınırlarını ve i32 çarpma taşmalarını kapsayan toplam **20 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1534/1534** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `99f42388f3562f04669e01818a39202ee8f698f0151f153acf5352e96f0fcc58`. Birleşik referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Adapter hashleri: debug `c0ccef8f56e8193f0ee8d76b66cee34ceb84310952f2f6bced3e9565c3fc3a36`, release `371d975bc049366bd9dc9f0d528455416d0fc508d885380fe5fa239d21082f7d`.

Yerel raporlar: `compare-t6yvipac` ve `compare-6_snmnk3` (math debug/release), `compare-tgx0qoj4` ve `compare-t1bsi_04` (birleşik debug/release), `verify-qesbmcro` (math) ve `verify-zwp9iban` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.mulAndTruncate` helper'ının pozitif ve sıfır olmayan denominator ile oluşturulan sabit Fraction alanları, Java i32 taşması ve truncating bölüm sonuçları içindir. Fraction factory hata yolları, diğer Fraction işlemleri ve bu helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
