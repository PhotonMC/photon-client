# P01.d — Shapes.rotateAttachFace initial overload referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, north shape'inden floor, wall ve ceiling attach face'leri için dört yatay yön map'i üreten `Shapes.rotateAttachFace(north, initial)` overload'ının Rust `ShapeRotation` alt kümesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:454-466`, `com/mojang/math/OctahedralGroup.java` ve `world/level/block/state/properties/AttachFace.java:5-8` incelendi. `MathOracle` doğrudan initial overload'ını çağırır; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla gözlenen yolları kapsar. Rust `ShapeRotation` enum'undaki diğer başlangıçlar ve Java'nın kalan `OctahedralGroup` değerleri ayrı dilimlerdir.

## Davranış sözleşmesi

- `Shapes.rotateAttachFace(north, initial)`, `FLOOR`, `WALL` ve `CEILING` anahtarlarıyla her biri dört yatay yön içeren bir map üretir.
- `WALL` map'i initial dönüşünü kullanır. `FLOOR`, `BLOCK_ROT_X_270.compose(initial)`; `CEILING` ise `BLOCK_ROT_Y_180.compose(BLOCK_ROT_X_90).compose(initial)` başlangıcıyla `rotateHorizontal` yolunu kullanır.
- Dış map sırası Java `AttachFace.values()` ile `FLOOR`, `WALL`, `CEILING`; iç map sırası Java `Direction` enum ile `NORTH`, `SOUTH`, `WEST`, `EAST` olarak gözlenir.
- Her iç map'in yatay yön bileşikleri Java `OctahedralGroup.compose` sırasını korur; shape koordinatları ve dolu hücre indisleri signed eksen dönüşümleriyle taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| identity | 1 | Asimetrik box ve `IDENTITY` initial |
| x-90 | 1 | Asimetrik box ve `ROT_90_X_NEG` initial |
| y-180 | 1 | Y ekseni yarım cube-grid box ve `ROT_180_FACE_XZ` initial |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_attach_face_initial"`, `box` ve `initial` alanlarını kullanır. Sonuç `faces` dizisinde attach face adı ile iç yön/shape snapshot dizisini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı.

Math debug/release raporları `compare-00tgiqc4` ve `compare-aylr4ozm`; birleşik debug/release raporları `compare-5bjkz31s` ve `compare-zfkba6ts` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java 26.2 `Shapes.rotateAttachFace(north, initial)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla FLOOR, WALL ve CEILING dış sırası ile iç dört yön map'lerinin bileşimleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Diğer `OctahedralGroup` değerleri ve attach-face dışındaki overload'lar ayrı dilimlerdir.
