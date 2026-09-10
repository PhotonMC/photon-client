# P01.a — Mth tamsayı yardımcıları doğrulaması

`src/math.rs`, Java `Mth` tamsayı bölme, pozitif modulo ve yuvarlama yardımcılarını Java'nın işaret ve taşma kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `floorDiv`, `positiveModulo`, `positiveCeilDiv` ve `roundToward` için 15 özgün senaryo hazırlandı.
- Negatif kalan ve bölen işaretleri, `Integer.MIN_VALUE`/`Long.MIN_VALUE` unary negation ve bölüm taşmaları ile sonuç çarpması taşmaları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1354/1354** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1856/1856** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `7bacfa4674e388b5f38f1b0a041e5531afe27e845d4374895c79e4fb56697a24`. Birleşik referans kimliği: `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7`.

Adapter hashleri: debug `45dccbceba09502c7c6c8e18ca105a628633653b7a27d8cb49b0aba1229bac62`, release `8274db44ba308fe51298d304e3f2dfbd62e8e2a353a0f84b806ee6d3643b26db`.

Yerel raporlar: `compare-z9ypl4_s` ve `compare-0xhgd0yz` (math debug/release), `compare-ote21q3t` ve `compare-r5nijsku` (birleşik debug/release), `verify-33997f0f` (math) ve `verify-2b2qykhl` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.floorDiv(int,int)`, `positiveModulo(int,int)`, `positiveCeilDiv(int,int)`, `positiveCeilDiv(long,long)`, `roundToward(int,int)` ve `roundToward(long,long)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, diğer Mth tamsayı overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
