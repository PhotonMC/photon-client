# P01.c — Direction unit-vectors referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.getUnitVec3i()`, `getUnitVec3()` ve `getUnitVec3f()` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:383-393` incelendi. Üç getter, Direction kurucusunda saklanan normalin sırasıyla i32 Vec3i, f64 Vec3 ve f32 Vector3fc görünümünü sunar.

## Davranış sözleşmesi

- DOWN, UP, NORTH, SOUTH, WEST ve EAST yönleri sırasıyla (0,-1,0), (0,1,0), (0,0,-1), (0,0,1), (-1,0,0) ve (1,0,0) bileşenlerini taşır.
- Vec3 görünümü f64, Vector3fc görünümü f32 bileşen bitleriyle karşılaştırılır; sıfır bileşenleri pozitif zero olarak gözlemlenir.
- Getter sonuçları yönün saklanan normalini değiştirmez.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_unit_vectors"` ve yön adı taşıyan `name` alanını içerir. Çıktı i32, f64 ve f32 normal vektör görünümlerini taşır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Direction `step()` integer helper'ı, Vec3 relative kullanımı, matrix rotation ve diğer Direction dönüşümleri ayrı davranış sözleşmeleridir.
