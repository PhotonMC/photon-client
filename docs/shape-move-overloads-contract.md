# P01.d — VoxelShape move overload'ları referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-move-overloads-parity.md)). Kapsam, `VoxelShape.move(Vec3)` ve `VoxelShape.move(Vec3i)` çağrılarının shape koordinatlarını scalar move yoluna taşımasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:73-95`, `world/phys/Vec3.java`, `core/Vec3i.java:61-75` ve `world/phys/shapes/Shapes.java:44-58` incelendi. `MathOracle`, her adımda gerçek Java overload'ını çağırır; beklenen koordinatları test içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `move(Vec3)` delta nesnesinin üç double alanını sırasıyla X, Y ve Z scalar delta'sı olarak iletir.
- `move(Vec3i)` delta nesnesinin üç int getter sonucunu double'a yükseltip aynı scalar move yoluna iletir. i32 aralığındaki değerler bu dönüşümde tam korunur.
- Empty shape iki overload'da da yeni offset listeleri üretmeden `Shapes.empty()` sonucuna döner.
- Dolu shape'in discrete hücre işgali değişmez; yalnızca X, Y ve Z koordinat düzlemlerine kendi delta'ları eklenir.
- Koordinat güncellemeleri Java double toplama sırasını ve signed zero/özel değer bitlerini korur. Sonuçlar toleranssız ham f64 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Vec3 | 1 | Asimetrik box ve negatif/pozitif double delta |
| Vec3i | 1 | Negatif koordinatlı box ve üç integer block delta'sı |
| empty | 1 | Empty shape üzerinde Vec3 ve Vec3i overload zinciri |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "single"` ve `steps` alanlarını kullanır. Scalar adım `op: "move"`, Vec3 overload'u `op: "move_vec3"` ve Vec3i overload'u `op: "move_block_pos"` taşır. Vec3 delta'sı `delta` altında f64 bit string'leri, Vec3i delta'sı aynı alan altında ondalık i32 string'leri taşır. Her adımın sonucu Java shape snapshot'ı olarak kaydedilir.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin bulunduğu güncel math referansı `d2ca721ca4ab536a692660afc465bae364927288154e3b38676eb7e5c86fa0f8` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `d2ca721ca4ab536a692660afc465bae364927288154e3b38676eb7e5c86fa0f8` kimliğiyle doğrulandı.

Math debug/release raporları `compare-2oxv9wmk` ve `compare-54arzdz4`; birleşik debug/release raporları `compare-3tby77d3` ve `compare-nscg_s6g` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
