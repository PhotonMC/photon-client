# P01.d — Shapes.rotateAll initial overload referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, north shape'inden altı yön map'i üreten `Shapes.rotateAll(north, initial, rotationCenter)` overload'ının Rust `ShapeRotation` alt kümesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:427-452`, `com/mojang/math/OctahedralGroup.java` ve `SymmetricGroup3.java` incelendi. `MathOracle` doğrudan üç parametreli overload'ı çağırır; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla seçilmiş merkez senaryolarını kapsar. Rust `ShapeRotation` enum'undaki diğer başlangıçlar ve Java'nın kalan `OctahedralGroup` değerleri ayrı dilimlerdir. Varsayılan ve yalnızca merkez alan overload'ları da ayrı kapsamdır.

## Davranış sözleşmesi

- `Shapes.rotateAll(north, initial, rotationCenter)`, `DOWN`, `UP`, `NORTH`, `SOUTH`, `WEST` ve `EAST` anahtarlarıyla bir `EnumMap` üretir.
- `NORTH` değeri `rotate(north, initial)` yolundan gelir ve Java'nın varsayılan blok merkezi `(0.5, 0.5, 0.5)` kullanır. Diğer yönler verilen `rotationCenter` ile döndürülür.
- `DOWN`, `UP`, `SOUTH`, `WEST` ve `EAST` bileşikleri sırasıyla `BLOCK_ROT_X_90`, `BLOCK_ROT_X_270`, `BLOCK_ROT_Y_180`, `BLOCK_ROT_Y_270` ve `BLOCK_ROT_Y_90` dönüşlerinin `initial` ile Java compose sırasındaki sonuçlarıdır.
- Java `Direction` enum sırası sonuç JSONL'ye `DOWN`, `UP`, `NORTH`, `SOUTH`, `WEST`, `EAST` olarak yansır; koordinat düzlemleri ve dolu hücre indisleri signed eksen permütasyonunu korur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| identity | 1 | Asimetrik box, `IDENTITY` initial ve özel merkez |
| x-90 | 1 | Asimetrik box, `ROT_90_X_NEG` initial ve özel merkez |
| y-180 | 1 | Y ekseni yarım cube-grid box, `ROT_180_FACE_XZ` initial ve özel merkez |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_all_initial"`, `box`, `initial` ve `center` alanlarını kullanır. Sonuç `shapes` dizisinde yön adı ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı.

Math debug/release raporları `compare-00tgiqc4` ve `compare-aylr4ozm`; birleşik debug/release raporları `compare-5bjkz31s` ve `compare-zfkba6ts` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, initial, rotationCenter)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla DOWN, UP, NORTH, SOUTH, WEST ve EAST yönlerinin enum sırası, bileşik dönüşleri ve özel merkez davranışı karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri, üç sentetik box senaryosunu ve verilen `rotationCenter` değerlerini kapsar. `NORTH` yolunun blok merkezini koruması ayrıca bu sözleşmede sabitlenmiştir. Rust `ShapeRotation` enum'undaki diğer başlangıç değerleri, Java'nın kalan `OctahedralGroup` değerleri ve diğer `rotateAll` overload'ları ayrı dilimlerdir.
