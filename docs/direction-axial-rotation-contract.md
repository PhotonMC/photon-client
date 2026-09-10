# P01.c — Direction axial-rotation referans sözleşmesi

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
