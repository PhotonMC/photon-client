# P01.d — AABB ofSize sözleşmesi

Bu dilim, Java `AABB.ofSize(Vec3, double, double, double)` statik üreticisinin kutu endpoint'lerini merkez ve üç eksen boyutundan üretmesini kapsar. Altı Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](aabb-of-size-parity.md).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:463-467` içindeki `ofSize` gövdesi ve `Vec3`'ün public final double koordinatları incelendi. `MathOracle` her adımda doğrudan `AABB.ofSize` çağırır; beklenen endpoint'leri oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- Her alt endpoint merkez koordinatından ilgili boyutun yarısı çıkarılarak üretilir.
- Her üst endpoint merkez koordinatına ilgili boyutun yarısı eklenerek üretilir.
- Altı ifade x, y, z sırasıyla değerlendirilir; bölme ve toplama/çıkarma Java double semantiğiyle yapılır.
- Üretilen endpoint'ler `AABB` constructor'ına verilir. Constructor her eksen çiftini Java `Math.min`/`Math.max` davranışıyla normalize eder; negatif boyutlar bu nedenle eksen uçlarını ters çevirebilir.
- Sıfır, signed zero, sonsuzluk ve NaN değerlerinin IEEE-754 double bitleri gözlenebilir çıktının parçasıdır.
- `ofSize` mevcut kutuyu değiştirerek yeni kutu snapshot'ı üretir; sonraki adımlar yeni kutu üzerinden çalışır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Birim kutu | 1 | `(0.5, 0.5, 0.5)` merkezi ve üç boyutta `1.0` |
| Asimetrik boyutlar | 1 | Negatif/pozitif merkez ve üç farklı pozitif boyut |
| Negatif boyutlar | 1 | Üç eksende negatif boyut ve constructor normalizasyonu |
| Sıfır boyut | 1 | Sıfır boyutlu kutu |
| Signed zero | 1 | Merkez ve boyutlarda pozitif/negatif sıfır |
| Özel değerler | 1 | Sonsuzluk ve NaN merkez/boyutları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "of_size"`, `center` Vec3 nesnesi ve `size_x`, `size_y`, `size_z` f64 bit string'leri bulunur. Adım, kutu snapshot'ı ekler.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

`ofSize` senaryolarını içeren math referans kimliği `c067c0c2f0322ab2784a92ca8072c3585d3c8c0367c9e6bce26190d0f22adf75` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](aabb-of-size-parity.md).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
