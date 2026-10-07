# P01.d — AABB BoundingBox köprüsü sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java `AABB.of(BoundingBox)` üreticisini ve üreticiye giren `BoundingBox(int, int, int, int, int, int)` sınır davranışını kapsar. Altı özgün Java davranış senaryosu hazırlandı ve Rust karşılığı [doğrulandı](#client-parity-doğrulaması).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:39-41` içindeki `of` üreticisi ile `world/level/levelgen/structure/BoundingBox.java:48-64` içindeki int constructor ve `:288-318` aralığındaki accessor'lar incelendi. `MathOracle` her adımda `BoundingBox` oluşturup doğrudan `AABB.of` çağırır; beklenen endpoint'leri yeniden hesaplamaz.

## Davranış sözleşmesi

- `BoundingBox` altı int endpoint'i saklar. Herhangi bir eksende `max < min` ise constructor o eksendeki iki değeri `Math.min`/`Math.max` ile sıralar; diğer eksenler de aynı constructor geçişinde kendi uçlarıyla bağımsız işlenir.
- `AABB.of` normalize edilmiş `minX`, `minY`, `minZ` değerlerini alt f64 endpoint'lerine taşır.
- `AABB.of` üst endpoint'lerini `maxX + 1`, `maxY + 1`, `maxZ + 1` Java int işlemiyle hesaplar ve sonucu double AABB constructor'ına verir. `Integer.MAX_VALUE + 1` bu aşamada `Integer.MIN_VALUE` olarak sarar.
- Son AABB constructor'ı uçları eksen bazında yeniden normalize eder; bu nedenle üst uç taşması, normal tamsayı aralığı veya aynı uç değeri sonuç snapshot'ında doğrudan gözlenir.
- Her üretici adımı önceki AABB değerini değiştirir ve yeni alt/üst endpoint snapshot'ı ekler.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Normal kutu | 1 | Sıralı negatif ve pozitif int endpoint'leri |
| Tüm eksenler ters | 1 | Constructor normalizasyonu |
| Seçili eksenler ters | 1 | X ve Z ters, Y sıralı |
| Integer maksimum taşması | 1 | Üç üst uçta `Integer.MAX_VALUE + 1` |
| Integer sınırları | 1 | `Integer.MIN_VALUE` ile `Integer.MAX_VALUE` aralığı |
| Üretici zinciri | 1 | İki ardışık `AABB.of(BoundingBox)` çağrısı |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "of_bounding_box"` ve altı decimal int string alanı taşıyan `box` nesnesi bulunur. Her adım sonrası AABB uçları f64 bit string olarak `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin math referans kimliği `5cb113d71b4ba206c07ea73382625520d811f4ecd4bef0769e15bdd55427a0b4` oldu. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/aabb.rs`, Java `BoundingBox` int sınırlarını normalize edip `AABB.of(BoundingBox)` üreticisinin üst uçlardaki Java int `+1` işlemini uygular. `examples/behavior_adapter.rs` bu yolu JSONL math senaryolarına bağlar.

## Referans ve davranış

Java 26.2 / protokol 776 / Java 25 referansı, `AABB.java:39-41` ve `BoundingBox.java:48-64,288-318` kaynak yollarıyla kullanıldı. Ayrıntılı davranış ve altı senaryonun girdileri [sözleşme belgesindedir](#sözleşme). Ters sınır senaryoları için referans classpath'ına sabitlenmiş `brigadier-1.3.10.jar` eklendi; bu yalnızca Java'nın constructor uyarı yolunun çalışabilmesi içindir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 474 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **474/474** math senaryosunda eşleşti.
- Altı yeni senaryoda normal sınırlar, tümü ve seçili eksenleri ters sınırlar, `Integer.MAX_VALUE + 1` taşması, iki tamsayı ucu ve ardışık üretici çağrıları karşılaştırıldı.
- Altı ailenin birleşik corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5cb113d71b4ba206c07ea73382625520d811f4ecd4bef0769e15bdd55427a0b4`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-_1vm5zsh` (math debug), `compare-emu4ivo7` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `BoundingBox` altı int endpoint constructor'ı, `AABB.of(BoundingBox)` ve mevcut AABB adım adaptörüyle sınırlıdır. VoxelShape clip tamamlandı; occlusion ve istemci hedefleme davranışları sonraki dilimlerdir.
