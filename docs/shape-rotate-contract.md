# P01.d — Shapes.rotate referans sözleşmesi

Bu dilimin **12 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-parity.md)). Kapsam, `VoxelShape` değerinin blok merkezine göre desteklenen signed eksen dönüşümleriyle döndürülmesini ve OR ile birleştirilmiş shape'in döndürülmesini içerir. `Shapes.rotateHorizontalAxis` çıktısı ayrı [sözleşme](shape-rotate-horizontal-axis-contract.md) ve [parity](shape-rotate-horizontal-axis-parity.md) belgelerinde; `Shapes.rotateAllAxis` çıktısı ayrı [sözleşme](shape-rotate-all-axis-contract.md) ve [parity](shape-rotate-all-axis-parity.md) belgelerinde; `Shapes.rotateHorizontal` çıktısı ayrı [sözleşme](shape-rotate-horizontal-contract.md) ve [parity](shape-rotate-horizontal-parity.md) belgelerinde; `Shapes.rotateAll` çıktısı ayrı [sözleşme](shape-rotate-all-contract.md) ve [parity](shape-rotate-all-parity.md) belgelerinde; merkez rotationPoint overload'ı ayrı [sözleşme](shape-rotate-center-contract.md) ve [parity](shape-rotate-center-parity.md) belgelerinde ele alınır. Shapes.rotateAttachFace sonucu ayrı [sözleşme](shape-rotate-attach-face-contract.md) ve [parity](shape-rotate-attach-face-parity.md) belgelerinde ele alınır. Shapes.rotateHorizontal initial sonucu ayrı [sözleşme](shape-rotate-horizontal-initial-contract.md) ve [parity](shape-rotate-horizontal-initial-parity.md) belgelerinde ele alınır. Shapes.rotateAll initial sonucu ayrı [sözleşme](shape-rotate-all-initial-contract.md) ve [parity](shape-rotate-all-initial-parity.md) belgelerinde ele alınır. Shapes.rotateAttachFace initial sonucu ayrı [sözleşme](shape-rotate-attach-face-initial-contract.md) ve [parity](shape-rotate-attach-face-initial-parity.md) belgelerinde ele alınır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:323-351`, `world/phys/shapes/DiscreteVoxelShape.java:24-45` ve `com/mojang/math/OctahedralGroup.java` incelendi. `MathOracle` doğrudan `Shapes.rotate` çağrısını yapar; beklenen kutuları yeniden hesaplamaz.

## Davranış sözleşmesi

- `Shapes.rotate(shape, rotation)`, shape'i varsayılan blok merkezi `(0.5, 0.5, 0.5)` çevresinde döndürür. Identity dönüşü shape geometrisini olduğu gibi döndürür.
- Dönüşüm, `OctahedralGroup` permütasyonunun output eksenlerini eski eksenlerden seçer. Signed eksenlerde koordinat düzlemleri ters sırada alınır ve her değer blok merkezine göre yansıtılır.
- Discrete voxel grid'in eksen boyutları aynı permütasyonla taşınır. Ters çevrilen output eksenlerinde dolu hücre indisleri yeni eksen boyutunun sonundan geriye eşlenir.
- Bu dilim identity, üç negatif 90 derece, üç 180 derece yüz dönüşü ve üç pozitif 90 derece dönüş adını kapsar. `rotate_join`, iki box'ı önce `OR` ile birleştirip aynı dönüş yoluna verir.
- Sonuç `toAabbs` sırasındaki kutular, empty durumu ve non-empty bounds alanlarıyla gözlenir. Box uçları f64 IEEE-754 bit dizgeleriyle taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| tek box dönüşleri | 10 | Identity, dokuz signed 90/180 derece dönüş |
| grid boyutu | 1 | Bir ekseni yarım olan box ve Y dönüşü |
| OR shape dönüşü | 1 | İki dolu hücrenin OR birleşimi ve Z dönüşü |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate"` veya `kind: "rotate_join"` kullanır. Sonuç shape snapshot'ı `empty`, Java `toAabbs` sırasındaki `boxes` ve non-empty durumda `bounds` alanlarını taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `20a0069e9085f30715b87aa3faae8a6ddc18d0b8a9e38233defa7ee4c21de6a8` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `20a0069e9085f30715b87aa3faae8a6ddc18d0b8a9e38233defa7ee4c21de6a8` kimliğiyle doğrulandı.

Math debug/release raporları `compare-dj0jpbgi` ve `compare-n_mw5890`; birleşik debug/release raporları `compare-wess7nxh` ve `compare-pqt_xf95` dizinlerindedir. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
