# P01.d — Shapes.rotateAttachFace referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-attach-face-parity.md)). Kapsam, north shape'inden floor, wall ve ceiling attach face'leri için dört yatay yönlü shape map'leri üreten varsayılan Shapes.rotateAttachFace(north) overload'ıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. world/phys/shapes/Shapes.java:454-466, com/mojang/math/OctahedralGroup.java ve world/level/block/state/properties/AttachFace.java incelendi. MathOracle doğrudan Shapes.rotateAttachFace çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca identity başlangıçlı tek argümanlı overload'ı kapsar. `initial` rotation alan overload ayrı [sözleşme](shape-rotate-attach-face-initial-contract.md) ve [parity](shape-rotate-attach-face-initial-parity.md) belgelerinde ele alınır; özel merkezli yön map'leri ve diğer OctahedralGroup değerleri ayrı dilimlerdir.

## Davranış sözleşmesi

- Shapes.rotateAttachFace(north), AttachFace.values() sırasındaki FLOOR, WALL, CEILING girdileri için dört yönlü map üretir.
- WALL map'i north shape'in varsayılan yatay dönüşlerini kullanır. FLOOR map'i BLOCK_ROT_X_270 başlangıç dönüşüyle, CEILING map'i ise BLOCK_ROT_Y_180 ile BLOCK_ROT_X_90 bileşiğinin başlangıç dönüşüyle aynı yatay yön yolunu kullanır.
- İç map'ler Java Direction enum sırasını korur; gözlenen yönler NORTH, SOUTH, WEST, EAST'tir. Her yön, ilgili attach face'in başlangıç dönüşü üzerine yatay yön bileşiğini uygular.
- Empty shape üç attach face ve dört yönün tamamında empty snapshot üretir. Bileşik dönüşlerde shape koordinat düzlemleri ve dolu hücre indisleri Java Shapes.rotate ile aynı signed eksen permütasyonu kurallarını izler.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Floor, wall ve ceiling bileşik dönüşlerini ayırt eden box |
| grid boyutu | 1 | Y ekseni yarım olan cube-grid box |
| empty shape | 1 | 1e-7 eşiğinin altında aralık |

Girdiler family: "math", operation: "shape", kind: "rotate_attach_face" ve box alanını kullanır. Sonuç faces dizisinde attach face adı ile iç shapes yön/snapshot dizisini taşır.

## Java yakalama

Depo kökünden:

Yakalama komutu: python3 tools/behavior/run.py capture --family math

Bu dilimdeki senaryoların bulunduğu güncel math referansı ee01fb100a4d138a2dc2967dc19d79f7e79a055fec252d406d23599850925ae3 kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı b251a60111607ad28c03f7396010655e88be4532d7dc3f048d4f3db054fce03e kimliğiyle doğrulandı.

Math debug/release raporları compare-v217glqz ve compare-snqwxxvj; birleşik debug/release raporları compare-ngv2kroy ve compare-r5i49tld dizinlerindedir, .data/runs/behavior/ altında bulunur. Ham girdiler ve Java çıktıları yalnızca .data/ altında tutulur.
