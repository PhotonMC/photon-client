# P01.d — Shapes.rotateAttachFace referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, north shape'inden floor, wall ve ceiling attach face'leri için dört yatay yönlü shape map'leri üreten varsayılan Shapes.rotateAttachFace(north) overload'ıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. world/phys/shapes/Shapes.java:454-466, com/mojang/math/OctahedralGroup.java ve world/level/block/state/properties/AttachFace.java incelendi. MathOracle doğrudan Shapes.rotateAttachFace çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca identity başlangıçlı tek argümanlı overload'ı kapsar. `initial` rotation alan overload ayrı [sözleşme](shape-rotate-attach-face-initial-conformance.md) ve [parity](shape-rotate-attach-face-initial-conformance.md) belgelerinde ele alınır; özel merkezli yön map'leri ve diğer OctahedralGroup değerleri ayrı dilimlerdir.

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

Bu dilimdeki senaryoların bulunduğu güncel math referansı 8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı 8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b kimliğiyle doğrulandı.

Math debug/release raporları compare-00tgiqc4 ve compare-aylr4ozm; birleşik debug/release raporları compare-5bjkz31s ve compare-zfkba6ts dizinlerindedir, .data/runs/behavior/ altında bulunur. Ham girdiler ve Java çıktıları yalnızca .data/ altında tutulur.


## Client-parity doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateAttachFace(north) yolunu floor, wall ve ceiling map'leriyle uygular; floor ve ceiling başlangıçları Java OctahedralGroup.compose sırasıyla bileştirir, iç map'leri NORTH/SOUTH/WEST/EAST sırasına taşır. examples/behavior_adapter.rs üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri üç attach face'i, her iç yön map'ini, bileşik dönüş yönünü ve enum sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: 8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b.

Birleşik referans kimliği: 8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b.

Yerel raporlar: compare-00tgiqc4 (math debug), compare-aylr4ozm (math release), compare-5bjkz31s (birleşik debug) ve compare-zfkba6ts (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca identity başlangıçlı tek argümanlı overload, üç sentetik box ve floor/wall/ceiling içindeki dört yatay yön map'iyle sınırlıdır. initial rotation overload'ı, özel merkezli yön map'leri ve diğer OctahedralGroup değerleri ayrı dilimlerdir.
