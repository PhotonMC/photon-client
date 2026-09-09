# P01.d — AABB sizes sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-sizes-parity.md)). Kapsam, Java `AABB.getXsize()`, `getYsize()`, `getZsize()` ve `getSize()` sorgularının normalize edilmiş kutu uçlarından kenar uzunluklarını ve üç uzunluğun aritmetik ortalamasını üretmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:267-284` incelendi. `MathOracle` dört getter'ı doğrudan çağırır; beklenen ölçüleri yeniden hesaplamaz.

## Davranış sözleşmesi

- `getXsize`, `getYsize` ve `getZsize` sırasıyla `max - min` çıkarımını kendi eksenlerinde yapar.
- `getSize`, üç kenar uzunluğunu `((x + y) + z) / 3.0` sırasıyla toplar ve böler.
- Kutunun uçları kurucuda normalize edildiği için ters verilen sonlu uçlar ölçü sorgularında non-negative uzunluklar üretir.
- IEEE-754 double çıkarma, toplama ve bölme bitleri; signed zero ve büyük değerlerdeki yuvarlama gözlenebilir çıktının parçasıdır.
- Sorgu kutuyu değiştirmez; sonuçlar her adımda `events` içinde taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Temel ölçü | 2 | Birim ve dikdörtgen kutular |
| Kurucu sınırı | 2 | Ters uçların normalizasyonu ve sıfır uzunluk |
| Sayısal ayrıntı | 2 | Signed zero ve büyük toplam yuvarlaması |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "sizes"` bulunur. Her sonuç `x_size`, `y_size`, `z_size` ve `size` alanlarını f64 bit string olarak taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Sizes senaryolarını içeren math referans kimliği `576e0822a567519f05faf72cba233f0591fa0c7e07cf41b43f37b88eecd0b61d` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `7868dda2fe1dd0a7fbb9e78e734b7ecf0ffc505e37763257267a72556cdbc671` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 400/400, birleşik corpus'ta 1485/1485 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
