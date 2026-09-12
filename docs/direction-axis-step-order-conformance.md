# P01.c — Direction axis-step-order referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`src/direction.rs` içindeki `axis_step_order`, Java `Direction.axisStepOrder(Vec3)` strict X/Z mutlak büyüklük karşılaştırmasını ve iki sabit eksen sırasını uygular. Rust adapter `direction_axis_step_order` JSONL operation'ı ile eksen listesini dışarı verir; Java oracle doğrudan Vec3 overload'unu çağırır.

Senaryolar işaretli büyüklükleri, eşitliği, sıfır/subnormal değerleri, sonsuzlukları ve NaN karşılaştırmalarını kapsar. Beklenen eksen dizileri Java oracle çıktısından alındı.

Java math capture kimliği `acde9c1776a4e0d57ac928e921d52cff658b64f5de0022efaf581073a7cf6200`, birleşik capture kimliği `71ba276d06d37c7bf75b45a4b43fb97b815d1ec4cc2152d79165d5425df2c9e0` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca axis-step-order sonucunu kapsar; diğer Direction bulma, dönüştürme ve entity bakış akışları ayrı davranış sözleşmeleridir.
