# P01.c — SectionPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed SectionPos long içindeki signed x, y ve z alanlarını Java `SectionPos.x(long)`, `y(long)` ve `z(long)` ile aynı i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API'lere bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, signed 22/20 bit uçları ve alternatif bit deseni doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1354/1354** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1856/1856** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1856 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `7bacfa4674e388b5f38f1b0a041e5531afe27e845d4374895c79e4fb56697a24`. Birleşik referans kimliği: `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7`.

Adapter hashleri: debug `f8fafbf2d2cd3ad40cd739a394f6d29f2575d4a9a3d7e85b6246579b4ce57682`, release `30554a139fc26f799697f0ab5c6ef3f2501d976b96dfa59d85ba7f2f9d7d6e96`.

Yerel raporlar: `compare-z9ypl4_s` (math debug), `compare-0xhgd0yz` (math release), `compare-ote21q3t` (birleşik debug), `compare-r5nijsku` (birleşik release) ve `verify-2b2qykhl` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından section x/y/z çıkarma ve signed sign extension yorumuyla sınırlıdır. SectionPos üreticileri, blok dönüşümleri, relative konumlar, stream codec tüketimi ve bu accessor'ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
