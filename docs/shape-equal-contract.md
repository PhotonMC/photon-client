# P01.d — Shapes.equal referans sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-equal-parity.md)). Kapsam, iki `VoxelShape` değerinin kapladığı hücreler bakımından eşitliğini ve iki empty shape'in eşit kabul edilmesini içerir. Signed eksen dönüşümleri ayrı [rotation sözleşmesinde](shape-rotate-contract.md) ele alınır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:375-377` ve `BooleanOp.java:3-21` incelendi. `MathOracle` `Shapes.equal` çağrısını doğrudan yapar; beklenen sonucu yeniden hesaplamaz.

## Davranış sözleşmesi

- `Shapes.equal(first, second)`, `NOT_SAME` BooleanOp birleşiminin dolu olup olmadığını sorar ve sonucu tersler.
- Karşılaştırma Java'nın shape hücreleri ve koordinat bölümlendirmesi üzerinden yapılır. Ayrı nesneler aynı hücreleri kaplıyorsa eşit olabilir.
- Herhangi bir hücre yalnızca bir shape'te doluysa sonuç `false` olur. Boyut farkı, kısmi örtüşme ve yalnızca sınırdan temas eden kutular bu yolu ayırır.
- İki shape de empty ise, koordinatları farklı olsa bile `NOT_SAME` birleşimi empty kalır ve sonuç `true` olur.
- Box uçları senaryolarda f64 IEEE-754 bit dizgeleriyle taşınır; `Shapes.box` eşiğinin altında kalan aralıklar empty shape üretir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| aynı birim kutu | 1 | Aynı box girdisinin iki kez verilmesi |
| aynı taşınmış kutu | 1 | Sıfırdan farklı koordinatlarda aynı box |
| farklı geometri | 3 | Boyut farkı, sınırdan temas ve kısmi örtüşme |
| iki empty shape | 1 | Eşik altı, farklı konumlardaki iki sliver |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "equal"` kullanır. Her sonuç yalnızca `equal` boolean alanını taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `72703f27997a3fbdb61c87ead93863a7a53806ebb3533f7e4c676f9a81591df9` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `e5d1fae4e50021d9e6e70de3182785f1dc68c1643d56f5a99d7e5aae91807e14` kimliğiyle doğrulandı.

Math debug/release raporları `compare-8rckp6vr` ve `compare-4eof6x0t`; birleşik debug/release raporları `compare-s3ki0jz5` ve `compare-fng7itjt` dizinlerindedir. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
