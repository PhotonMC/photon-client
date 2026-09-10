# P01.a — Mth mulAndTruncate doğrulaması

`src/math.rs`, Java `Mth.mulAndTruncate(Fraction, int)` işlemini Fraction'ın numerator/denominator alanları üzerinden i32 çarpma ve sıfıra doğru bölüm sırasıyla taşır. `examples/behavior_adapter.rs` sonucu signed decimal i32 olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Pozitif ve negatif kesirleri, sıfır/negatif factor değerlerini, indirgenmemiş Fraction alanlarını, numerator/denominator sınırlarını ve i32 çarpma taşmalarını kapsayan toplam **20 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1723/1723** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2225/2225** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `efa8006dabc939417f5dae848830a2b524d131c4014d44a262a55fc501c1ba27`. Birleşik referans kimliği: `07898354b4bb5d4f13d9fb4841bbe80cf32e162852f5635cbf2013b0de73a161`.

Adapter hashleri: debug `ccc9db6aea47eb0a6ccfbc60596e5c667094bdc82d22cc1ff9bc4f4ede271b76`, release `404145a1cd1726a8e97c0930477be188dbe357582c941fe006671440dec0f070`.

Yerel raporlar: `compare-vdcnovbh` ve `compare-4w3_lr0t` (math debug/release), `compare-c5b8xhs4` ve `compare-mdqcgmw7` (birleşik debug/release), `verify-f0selcq4` (math) ve `verify-r95rgxgt` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.mulAndTruncate` helper'ının pozitif ve sıfır olmayan denominator ile oluşturulan sabit Fraction alanları, Java i32 taşması ve truncating bölüm sonuçları içindir. Fraction factory hata yolları, diğer Fraction işlemleri ve bu helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
