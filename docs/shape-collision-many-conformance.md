# P01.d — Shapes.collide çoklu shape sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `Shapes.collide` çağrısının aynı hareketli `AABB` ile birden fazla `VoxelShape` girdisini verilen sırada işlemesini içerir. Entity hareket sırası, step-up çözümü ve yüz/occlusion sonraki kapsamdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:234-244`, `VoxelShape.java:252-307` ve `AABB.java:86-105` incelendi. `MathOracle` doğrudan `Shapes.collide` çağırır; beklenen mesafeyi yeniden hesaplamaz.

## Davranış sözleşmesi

- Shape'ler iterable sırasıyla işlenir; her shape'in sonucu sonraki shape'e başlangıç distance'ı olur.
- Her shape işlenmeden önce mutlak distance `1e-7` değerinden küçükse sonuç `0.0` olarak erken döner.
- Empty shape mevcut distance'ı değiştirmez. Shape listesi boşsa başlangıç distance'ı döner.
- Bu dilimde liste elemanları tek `Shapes.box` sonucudur; birleşik/multi-box shape ve dünya collider toplama ayrı kapsamdır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Pozitif/negatif en yakın shape | 2 | Uzak shape'ten sonra daha yakın shape ile distance daraltma |
| Empty/erken dönüş | 2 | Empty ilk shape ve sıfır başlangıç distance'ı |
| Boş iterable | 1 | Shape listesi boşken başlangıç distance'ı |

`family: "math"`, `operation: "shape"`, `kind: "collide_many"` girdileridir. `axis` `x`, `y` veya `z`; `shapes` f64 bit string uçlu box nesneleri dizisi; `moving` AABB box nesnesi; `distance` f64 bit string'dir. Sonuç `distance` alanında f64 bit string taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Çoklu collision senaryolarını içeren math referans kimliği `14e57e20448a185606b7554345c087616634381f5b7865b345738ceea2dac811` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 366/366, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java `Shapes.collide` döngüsünü aynı moving AABB ve distance ile shape slice'ı üzerinde verilen sırada uygular. `examples/behavior_adapter.rs`, `shapes` dizisini, moving AABB'yi, ekseni ve başlangıç distance'ını JSONL'e bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 366 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **366/366** math senaryosunda eşleşti.
- Beş yeni senaryoda pozitif/negatif yönde ardışık daraltma, empty shape, sıfır distance erken dönüşü ve boş iterable sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `14e57e20448a185606b7554345c087616634381f5b7865b345738ceea2dac811`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-tdfdd7rh` (math debug), `compare-piprh3re` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, tek kutudan üretilen shape'lerin `Shapes.collide` iterable döngüsüyle sınırlanmıştır. Birleşik/multi-box shape, face/occlusion, entity collider toplama ve hareket çözüm sırası sonraki dilimlerdir.
