# P01.d — Shapes.rotate referans sözleşmesi

Bu dilimin **12 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-parity.md)). Kapsam, `VoxelShape` değerinin blok merkezine göre desteklenen signed eksen dönüşümleriyle döndürülmesini ve OR ile birleştirilmiş shape'in döndürülmesini içerir. `Shapes.rotateHorizontalAxis` çıktısı ayrı [sözleşme](shape-rotate-horizontal-axis-contract.md) ve [parity](shape-rotate-horizontal-axis-parity.md) belgelerinde; `Shapes.rotateAllAxis` çıktısı ayrı [sözleşme](shape-rotate-all-axis-contract.md) ve [parity](shape-rotate-all-axis-parity.md) belgelerinde ele alınır.

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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `b8df21c506d300a30947ce77e21841210f33991f41bd8bbdf4dd4a1ab775a343` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `b624d0ee4bd5155be0aa8e190a91cc25d84867d5c6f605db7c91ad8c599196f2` kimliğiyle doğrulandı.

Math debug/release raporları `compare-pxfrslbe` ve `compare-le8qojd3`; birleşik debug/release raporları `compare-ag226u45` ve `compare-8116vkuu` dizinlerindedir. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
