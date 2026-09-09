# P01.c — ChunkPos blok koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` blok koordinatı yardımcılarını aynı i32 kaydırma ve toplama kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik x/z/offset girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Minimum, orta, maksimum ve offset'li koordinatlar x/z sırasını korudu; negatif chunk koordinatları, shift taşması ve offset toplama taşması Java i32 sarma davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1019/1019** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1521 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `5df2c7aa290a2d3deb2e318553a41a27dbe6b08b22e1d31b68362eedc36e2108`. Birleşik referans kimliği: `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406`.

Yerel raporlar: `compare-_lwapdio` (math debug), `compare-ed7p25g6` (math release), `compare-uh_quimp` (birleşik debug), `compare-233_yr3a` (birleşik release) ve `verify-wqacm20n` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'un blok x/z alt-orta-üst uçlarını ve offset'li blok koordinatlarını kapsar. Region koordinatları, `BlockPos` üretimi, contains/distance/range yardımcıları, stream/JSON codec'leri ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
