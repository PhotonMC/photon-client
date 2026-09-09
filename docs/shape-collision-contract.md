# P01.d — VoxelShape collision sözleşmesi

Bu dilimin **8 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-collision-parity.md)). Kapsam, tek bir `VoxelShape` ile `AABB` hareketinin Java `VoxelShape.collide` çağrısında x/y/z eksenlerinde sınırlanmasını içerir. Çoklu shape iterasyonu ayrı [sözleşmede](shape-collision-many-contract.md) kapsanır; entity hareket sırası ve step-up çözümü sonraki kapsamdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:252-307`, `Shapes.java:237-250` ve `AABB.java:86-105` incelendi. `MathOracle` doğrudan `VoxelShape.collide` çağırır; beklenen mesafeyi yeniden hesaplamaz.

## Davranış sözleşmesi

- Empty shape hareket mesafesini değiştirmeden döndürür. Non-empty shape için mutlak mesafe `1e-7` değerinden küçükse sonuç `0.0` olur.
- Hareket ekseni tarama eksenidir; diğer iki eksendeki hareketli AABB aralığı shape hücreleriyle örtüşmüyorsa shape çarpışmaya katılmaz.
- Pozitif harekette ilk dolu hücrenin alt düzlemi ile hareketli kutunun üstü arasındaki mesafe; negatif harekette ilk dolu hücrenin üst düzlemi ile hareketli kutunun altı arasındaki mesafe kullanılır. Java'nın `±1e-7` kabulü ve başlangıç distance sınırı korunur.
- Moving AABB uçları Java kurucusundaki eksen bazlı `min`/`max` normalizasyonundan sonra kullanılır. Bu dilimde shape girdisi tek `Shapes.box` sonucudur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Pozitif/negatif x | 2 | Tam bloktan önce ve sonra hareket |
| Eksen permütasyonu | 2 | Pozitif y ve negatif z hareketi |
| Filtreleme/eşik | 4 | Ortogonal boşluk, sıfır distance, empty shape ve sınırlı distance |

`family: "math"`, `operation: "shape"`, `kind: "collide"` girdileridir. `axis` `x`, `y` veya `z`; `shape` ve `moving` f64 bit string uçlu box nesneleri; `distance` f64 bit string'dir. Sonuç `distance` alanında f64 bit string taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Tek shape collision senaryolarını içeren math referans kimliği `b31620fa93e0533f6498b86deedf056a7e4dda293e4fa80ae354dd69541d1ebd` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `79c0a319fee88ab66551f0d6e482274fbf9e528c52ae097bb43853784b4e846a` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri tek shape math referansında 361/361, güncel birleşik corpus'ta 1052/1052 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
