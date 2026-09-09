# P01.d — Shapes.rotateAll initial overload referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-all-initial-parity.md)). Kapsam, north shape'inden altı yön map'i üreten `Shapes.rotateAll(north, initial, rotationCenter)` overload'ının Rust `ShapeRotation` alt kümesidir.

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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `a96f6ac711181f55389712d207cddb3ac84e94a0b65b0aa592874d653bb8d9fd` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `b23e0bd7aa5f9df5baa9862e79e09f08f309c7b06b36028da91d501f3ddae578` kimliğiyle doğrulandı.

Math debug/release raporları `compare-fu6yzw05` ve `compare-jp_d2q8j`; birleşik debug/release raporları `compare-ixe0azli` ve `compare-k8ldje4k` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
