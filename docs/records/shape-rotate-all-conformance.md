# P01.d — Shapes.rotateAll referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, north shape'inden altı yön için shape map'i üreten Java yardımcı metodunun varsayılan overload'udur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:427-452` ve `com/mojang/math/OctahedralGroup.java` içindeki blok dönüş alias'ları incelendi. `MathOracle` doğrudan `Shapes.rotateAll` çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca `rotateAll(north)` overload'ını kapsar. Özel merkez overload'ı ayrı [sözleşme](shape-rotate-all-center-conformance.md) ve [parity](shape-rotate-all-center-conformance.md) belgelerinde ele alınır. `initial` ve `rotationCenter` alanlarını birlikte alan overload ayrı [sözleşme](shape-rotate-all-initial-conformance.md) ve [parity](shape-rotate-all-initial-conformance.md) belgelerinde ele alınır. `rotateHorizontal` ve `rotateAttachFace` yön map'leri ayrı dilimlerdir.

## Davranış sözleşmesi

- `Shapes.rotateAll(north)` varsayılan olarak `IDENTITY` başlangıç dönüşü ve blok merkezi `(0.5, 0.5, 0.5)` kullanır.
- NORTH anahtarındaki değer özgün shape'tir; EAST, SOUTH, WEST, UP ve DOWN anahtarları sırasıyla `BLOCK_ROT_Y_90`, `BLOCK_ROT_Y_180`, `BLOCK_ROT_Y_270`, `BLOCK_ROT_X_270` ve `BLOCK_ROT_X_90` dönüşleriyle üretilir.
- Java `EnumMap` direction enum sırasını korur; sonuç JSONL'de DOWN, UP, NORTH, SOUTH, WEST, EAST olarak gözlenir. Empty shape altı anahtarda da empty snapshot üretir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Altı dönüşü ayırt eden box |
| grid boyutu | 1 | Y ekseni yarım olan cube-grid box |
| empty shape | 1 | `1e-7` eşiğinin altında aralık |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_all"` kullanır. Sonuç `shapes` dizisinde `direction` ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı.

Math debug/release raporları `compare-00tgiqc4` ve `compare-aylr4ozm`; birleşik debug/release raporları `compare-5bjkz31s` ve `compare-zfkba6ts` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java 26.2 `Shapes.rotateAll(north)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar. Özel merkez overload sonucu ayrı [parity belgesinde](shape-rotate-all-center-conformance.md) doğrulanır; başlangıç ve özel merkez overload sonucu ayrı [initial parity belgesinde](shape-rotate-all-initial-conformance.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli altı yön map'iyle sınırlıdır. Özel merkez overload ayrı [rotateAll merkez parity belgesinde](shape-rotate-all-center-conformance.md) doğrulanır; başlangıç ve özel merkez overload ayrı [rotateAll initial parity belgesinde](shape-rotate-all-initial-conformance.md) doğrulanır; `rotateHorizontal`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
