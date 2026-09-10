# P01.d — Shapes.rotate merkez overload referans sözleşmesi

Bu dilimin **4 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-center-parity.md)). Kapsam, bir shape'i verilen `OctahedralGroup` dönüşü ve özel `Vec3 rotationPoint` ile döndüren Java overload'udur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:323-372`, `world/phys/shapes/DiscreteVoxelShape.java:24-45` ve `com/mojang/math/OctahedralGroup.java` incelendi. `MathOracle` doğrudan `Shapes.rotate(shape, rotation, rotationPoint)` çağrısını yapar; beklenen shape kutuları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca `rotationPoint` overload'ını kapsar. Varsayılan blok merkezli dönüşler ve bunları kullanan axis/direction map yardımcıları ayrı parity belgelerinde tutulur.

## Davranış sözleşmesi

- Identity dönüşünde Java merkezi okumadan özgün shape'i döndürür; özel merkez identity için gözlenebilir bir fark oluşturmaz.
- Identity dışı dönüşte discrete hücrelerin eksen permütasyonu ve terslenmesi dönüş grubuna göre yapılır. Koordinat düzlemleri, output ekseninin kaynak düzlemindeki `rotationPoint` bileşeninden output ekseninin bileşenine taşınır.
- Terslenen eksende koordinat listesi tersten okunur ve her değer kaynak merkez etrafında yansıtılır; terslenmeyen eksende sıra korunur ve merkez farkı eklenir. Hücre indisleri koordinat dönüşüyle aynı eksen permütasyonu ve tersleme sırasını izler.
- Box shape'in blok merkezli özel dönüş optimizasyonu özel merkezde kullanılmaz; sonuç snapshot'ı `empty`, `boxes` ve varsa `bounds` alanlarıyla gözlenir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Y dönüşü | 1 | Asimetrik box ve eksen bileşenleri farklı merkez |
| X dönüşü | 1 | Unit cube ve özel merkez |
| Z dönüşü | 1 | Y ekseni yarım grid ve özel merkez |
| Identity | 1 | Özel merkez verilmesine rağmen değişmeyen shape |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_center"`, `rotation`, `box` ve `center` alanlarını kullanır. Sonuç tek shape snapshot'ıdır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea` kimliğiyle doğrulandı.

Math debug/release raporları `compare-xljwmsb7` ve `compare-s_oqvkq3`; birleşik debug/release raporları `compare-uf5y7y3a` ve `compare-wrj9a6ex` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
