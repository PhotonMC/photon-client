# P01.c — Koordinat dönüşümleri referans sözleşmesi — [client-parity] kapsam kaydı

Bu dilimin **57 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](math-conformance.md)). Önceki 238 math senaryosunun girdisi ve sonucu değişmedi. Çift duyarlıklı overload bu int koordinat dilimine dahil değildir; ayrı [SectionPos double koordinat sözleşmesinde](sectionpos-double-coordinates-conformance.md) doğrulanır. `sectionRelativePos` paketi ve kalan SectionPos/BlockPos yardımcıları bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `core/SectionPos.java` içindeki `blockToSectionCoord(int)`, `sectionToBlockCoord(int)` ve `sectionRelative(int)` incelendi. Adaptör doğrudan orijinal metotları çağırır. Yeni bağımlılık eklenmedi.

## Davranış sözleşmesi

- `blockToSectionCoord` aritmetik `>> 4` kullanır; negatif bloklar tabana doğru yuvarlanır (`-1 → -1`, `-16 → -1`, `-17 → -2`).
- `sectionToBlockCoord` `<< 4` kullanır; taşma Java int sarmasıdır.
- `sectionRelative` `& 15` kullanır; sonuç her zaman `0–15` aralığındadır (`-1 → 15`).
- Sonuçlar işaretli ondalık string olarak taşınır; hata kategorisi yoktur.

## Senaryolar ve JSONL

19 koordinat × 3 fonksiyon = 57 senaryo: sıfır, 15/16/17 ve 31/32/33 komşuları, negatifler, i32 uçları ve section sınırları (33554431, -33554432, 2097151, -2097152).

`family: "math"`, `operation: "coord"` ortak girdilerdir. `fn` ve işaretli ondalık `value` taşınır; sonuç `value` alanıdır.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

295 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `9406952e0caa205e06c297a190684c65b8ba34c0e5b3abd2286b37c2c88fd88b`. Önceki 238 girdisi korunur. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Toplam 736 Java senaryosu yakalandı ve doğrulandı. Birleşik referans kimliği: `d8b68459939231e94c8dfd585eebb462329264c3d614fa7006d8bfea616461c3`. Eski kayıtlar değiştirilmedi.

Beklenen çıktılar Rust'a göre değiştirilmedi.
