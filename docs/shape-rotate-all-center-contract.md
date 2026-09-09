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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `ea03d29d513eca99de7f40ea68c6623f22649f5ce9a4d6dc4ef0a5ac02cb63f3` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `716f0b454b3848994e9afd84c60130e20d821311c22b7c1e80590008852fda52` kimliğiyle doğrulandı.

Math debug/release raporları `compare-8dsunddp` ve `compare-zvv3a_3_`; birleşik debug/release raporları `compare-9k3hskia` ve `compare-yw9njwwe` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
