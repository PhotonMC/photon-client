# P01.c — Direction axial-rotation referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.getClockWise(Axis)` ve `getCounterClockWise(Axis)` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:179-253` incelendi. Eksenli çağrılar, seçilen eksene paralel iki yönü korur ve kalan dört yönü eksen özelindeki dönüş tablosuna verir.

## Davranış sözleşmesi

- X ekseni dönüşleri DOWN/UP/NORTH/SOUTH çevrimini, Y ekseni dönüşleri yatay NORTH/EAST/SOUTH/WEST çevrimini, Z ekseni dönüşleri DOWN/UP/WEST/EAST çevrimini kullanır.
- Saat yönü ve karşı saat yönü tabloları birbirinin tersidir.
- Eksenle hizalı yönlerde clockwise ve counter-clockwise sonucu giriş yönünün aynısıdır.
- Altı yönün her biri X, Y ve Z eksenleriyle ayrı ayrı karşılaştırılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axial_rotation"`, `name` yön adını ve `axis` (`x`, `y` veya `z`) alanını taşır. Çıktı clockwise ve counter-clockwise yön snapshot'larını içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Eksen verilmeden `getClockWise()`/`getCounterClockWise()` çağrıları, matrix rotation, entity bakış yönü ve diğer Direction dönüşümleri ayrı davranış sözleşmeleridir.


## Client-parity doğrulaması

`src/direction.rs` içindeki `Direction::clockwise` ve `counter_clockwise`, Java eksen özel dönüş tablolarını ve hizalı yön kısa yolunu uygular. Rust adapter `direction_axial_rotation` JSONL operation'ı ile iki sonucu dışarı verir; Java oracle iki enum metodunu doğrudan çağırır.

On sekiz yön/eksen çifti Java oracle çıktısıyla karşılaştırılır.

Java math capture kimliği `b281ec0375183bbbdf0f5b15a1313103c7bd368f271b87a176a54c3b2c7d3786`, birleşik capture kimliği `41d6c2ceb63f7b055b5ab39658bf532208416282dd760be52177ee330023666b` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca eksenli clockwise/counter-clockwise yolları kapsar; eksensiz dikey hata yolları ve matrix rotation ayrı davranış sözleşmeleridir.
