# P01.d — Shapes.rotateAll merkez overload referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-all-center-parity.md)). Kapsam, bir north shape'inden altı yön map'i üreten ve dönüşlerde özel `Vec3 rotationCenter` kullanan `Shapes.rotateAll` overload'ıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:427-452` ve `com/mojang/math/OctahedralGroup.java` içindeki blok dönüş alias'ları incelendi. `MathOracle` doğrudan `Shapes.rotateAll(north, rotationCenter)` çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca özel merkez alan iki argümanlı overload'ı kapsar. `initial` rotation alan üç argümanlı overload ve `rotateAttachFace` ayrı dilimlerdir.

## Davranış sözleşmesi

- `Shapes.rotateAll(north, rotationCenter)`, `IDENTITY` başlangıç dönüşüyle `rotateAll(north, IDENTITY, rotationCenter)` yoluna gider.
- Java `EnumMap` direction enum sırasını korur; sonuç yönleri DOWN, UP, NORTH, SOUTH, WEST, EAST olarak gözlenir.
- NORTH girdisi başlangıç dönüşü olan `IDENTITY` üzerinden özel merkezi kullanmadan özgün shape olarak döner. Diğer yönler, ilgili blok dönüşüyle özel merkez etrafında döndürülür: DOWN `ROT_90_X_NEG`, UP `ROT_90_X_POS`, SOUTH `ROT_180_FACE_XZ`, WEST `ROT_90_Y_POS`, EAST `ROT_90_Y_NEG`.
- Sonuç `shapes` dizisindeki yön ve shape snapshot çiftlerini taşır; empty shape tüm yönlerde korunur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Altı yönü ayırt eden box ve farklı merkez bileşenleri |
| grid boyutu | 1 | Y ekseni yarım olan box ve özel merkez |
| empty shape | 1 | Eşik altı aralık ve özel merkez |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_all_center"`, `box` ve `center` alanlarını kullanır. Sonuç `shapes` dizisinde `direction` ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `7d929c43d190008e7958b13c5cd6ed1ecfd727fb2477efae2e307b70d7888deb` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `79c0a319fee88ab66551f0d6e482274fbf9e528c52ae097bb43853784b4e846a` kimliğiyle doğrulandı.

Math debug/release raporları `compare-yuxqj_ix` ve `compare-fpos2gkp`; birleşik debug/release raporları `compare-witt8p3m` ve `compare-pabdz7bu` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
