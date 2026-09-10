# P01.a — Mth tamsayı temel yardımcıları doğrulaması

`src/math.rs`, Java `Mth` mutlak değer, tamsayı clamp, katlık ve int açı sarmalama yardımcılarını Java'nın sınır ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `abs`, `clamp`, `isMultipleOf` ve `wrapDegrees(int)` için 25 özgün senaryo hazırlandı.
- `Integer.MIN_VALUE`, `Long.MIN_VALUE`, ters clamp sınırları, negatif katlık ve `[-180, 180)` açı uçları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1723/1723** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2225/2225** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `efa8006dabc939417f5dae848830a2b524d131c4014d44a262a55fc501c1ba27`. Birleşik referans kimliği: `07898354b4bb5d4f13d9fb4841bbe80cf32e162852f5635cbf2013b0de73a161`.

Adapter hashleri: debug `ccc9db6aea47eb0a6ccfbc60596e5c667094bdc82d22cc1ff9bc4f4ede271b76`, release `404145a1cd1726a8e97c0930477be188dbe357582c941fe006671440dec0f070`.

Yerel raporlar: `compare-vdcnovbh` ve `compare-4w3_lr0t` (math debug/release), `compare-c5b8xhs4` ve `compare-mdqcgmw7` (birleşik debug/release), `verify-f0selcq4` (math) ve `verify-r95rgxgt` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.abs(int)`, `clamp(int,int,int)`, `clamp(long,long,long)`, `isMultipleOf(int,int)` ve `wrapDegrees(int)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, açı ve tamsayı overload'larının geri kalanı ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
