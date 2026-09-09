# P01.d — Shapes.collide çoklu shape sözleşmesi

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-collision-many-parity.md)). Kapsam, Java `Shapes.collide` çağrısının aynı hareketli `AABB` ile birden fazla `VoxelShape` girdisini verilen sırada işlemesini içerir. Entity hareket sırası, step-up çözümü ve yüz/occlusion sonraki kapsamdır.

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

Çoklu collision senaryolarını içeren math referans kimliği `14e57e20448a161506b7554345c087616634381f5b7865b345738ceea2dac811` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `f13f272d7c3d33b3fc203feb94bab869310a7fe2c99a8b47d53c195952394c06` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 366/366, birleşik corpus'ta 976/976 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
