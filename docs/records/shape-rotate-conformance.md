# P01.d — Shapes.rotate referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **12 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `VoxelShape` değerinin blok merkezine göre desteklenen signed eksen dönüşümleriyle döndürülmesini ve OR ile birleştirilmiş shape'in döndürülmesini içerir. `Shapes.rotateHorizontalAxis` çıktısı ayrı [sözleşme](shape-rotate-horizontal-axis-conformance.md) ve [parity](shape-rotate-horizontal-axis-conformance.md) belgelerinde; `Shapes.rotateAllAxis` çıktısı ayrı [sözleşme](shape-rotate-all-axis-conformance.md) ve [parity](shape-rotate-all-axis-conformance.md) belgelerinde; `Shapes.rotateHorizontal` çıktısı ayrı [sözleşme](shape-rotate-horizontal-conformance.md) ve [parity](shape-rotate-horizontal-conformance.md) belgelerinde; `Shapes.rotateAll` çıktısı ayrı [sözleşme](shape-rotate-all-conformance.md) ve [parity](shape-rotate-all-conformance.md) belgelerinde; merkez rotationPoint overload'ı ayrı [sözleşme](shape-rotate-center-conformance.md) ve [parity](shape-rotate-center-conformance.md) belgelerinde ele alınır. Shapes.rotateAttachFace sonucu ayrı [sözleşme](shape-rotate-attach-face-conformance.md) ve [parity](shape-rotate-attach-face-conformance.md) belgelerinde ele alınır. Shapes.rotateHorizontal initial sonucu ayrı [sözleşme](shape-rotate-horizontal-initial-conformance.md) ve [parity](shape-rotate-horizontal-initial-conformance.md) belgelerinde ele alınır. Shapes.rotateAll initial sonucu ayrı [sözleşme](shape-rotate-all-initial-conformance.md) ve [parity](shape-rotate-all-initial-conformance.md) belgelerinde ele alınır. Shapes.rotateAttachFace initial sonucu ayrı [sözleşme](shape-rotate-attach-face-initial-conformance.md) ve [parity](shape-rotate-attach-face-initial-conformance.md) belgelerinde ele alınır.

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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı.

Math debug/release raporları `compare-00tgiqc4` ve `compare-aylr4ozm`; birleşik debug/release raporları `compare-5bjkz31s` ve `compare-zfkba6ts` dizinlerindedir. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/photon-core/src/shape.rs`, Java 26.2 `Shapes.rotate` yolunun blok merkezli signed eksen permütasyonunu, koordinat düzlemlerinin merkez etrafında yansıtılmasını ve discrete dolu hücre indislerinin taşınmasını uygular. `examples/behavior_adapter.rs` 12 yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotateHorizontalAxis` sonucu ayrı [axis parity belgesinde](shape-rotate-horizontal-axis-conformance.md), `Shapes.rotateAllAxis` sonucu [all-axis parity belgesinde](shape-rotate-all-axis-conformance.md), `Shapes.rotateHorizontal` sonucu [horizontal parity belgesinde](shape-rotate-horizontal-conformance.md), `Shapes.rotateAll` sonucu [all-direction parity belgesinde](shape-rotate-all-conformance.md), merkez overload sonucu [merkez rotation parity belgesinde](shape-rotate-center-conformance.md), Shapes.rotateAttachFace sonucu [attach-face parity belgesinde](shape-rotate-attach-face-conformance.md), Shapes.rotateHorizontal initial sonucu [initial parity belgesinde](shape-rotate-horizontal-initial-conformance.md), Shapes.rotateAll initial sonucu [all-direction initial parity belgesinde](shape-rotate-all-initial-conformance.md), Shapes.rotateAttachFace initial sonucu [attach-face initial parity belgesinde](shape-rotate-attach-face-initial-conformance.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- On tek-box dönüşü; identity, negatif/pozitif 90 derece dönüşler ve üç 180 derece yüz dönüşünü karşılaştırdı.
- Grid boyutu ve OR ile birleştirilmiş hücre senaryoları, yalnızca koordinat sınırlarını değil discrete hücre dönüşümünü ve `toAabbs` sırasını da karşılaştırdı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt identity ve `OctahedralGroup` içindeki dokuz non-identity dönüş adı ile box/OR shape girdileriyle sınırlıdır. Merkez parametreli overload ayrı [merkez rotation parity belgesinde](shape-rotate-center-conformance.md) doğrulanır; diğer shape kaynakları ve world face/collider toplama bu dilimin kapsamı değildir.
