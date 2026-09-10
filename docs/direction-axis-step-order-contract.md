# P01.c — Direction axis-step-order referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.axisStepOrder(Vec3)` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:379-381` incelendi. Yöntem yalnızca hareket vektörünün X ve Z bileşenlerinin mutlak büyüklüklerini karşılaştırır; Y ekseni her iki sonuç sırasının başında kalır.

## Davranış sözleşmesi

- `abs(x) < abs(z)` ise sıra `y`, `z`, `x` olur.
- Eşit büyüklüklerde strict karşılaştırma false olduğu için sıra `y`, `x`, `z` olur.
- NaN içeren karşılaştırmalarda Java karşılaştırması false üretir ve `y`, `x`, `z` sırası seçilir; sonsuzluklar da normal mutlak büyüklük karşılaştırmasına girer.
- Y bileşeni hesaplamada kullanılmaz ancak Vec3 girdisi oracle çağrısında tam olarak kurulur.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_step_order"` ve IEEE-754 f64 `x`, `y`, `z` bileşenlerini 16 hexadecimal bit olarak taşır. Çıktı üç eksen adından oluşan `axes` dizisidir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Direction seçimi, entity bakış sıralaması, random yön, codec'ler ve hareket vektörünün diğer yardımcıları bu dilime dahil değildir.
