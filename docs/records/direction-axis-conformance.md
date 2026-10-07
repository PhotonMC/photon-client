# P01.c — Direction axis-direction referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.fromAxisAndDirection(Axis, AxisDirection)` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:295-300` incelendi. Yöntem eksen ve eksen yönü enum çiftini doğrudan altı kardinal yöne çevirir.

## Davranış sözleşmesi

- X pozitif `east`, X negatif `west` yönünü seçer.
- Y pozitif `up`, Y negatif `down` yönünü seçer.
- Z pozitif `south`, Z negatif `north` yönünü seçer.
- Her sonuç, yönün adını, data değerlerini, eksenini, karşıtını ve step bileşenlerini taşıyan enum snapshot'ı olarak gözlenir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis"`, `axis` (`x`, `y`, `z`) ve `direction` (`positive`, `negative`) alanlarını içerir. Çıktı yön snapshot'ıdır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Yönün yaklaşıksal seçimi, entity bakış yönleri, random seçim, codec'ler ve quaternion dönüşümleri bu dilime dahil değildir.


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `from_axis_and_direction`, Java `Direction.fromAxisAndDirection` eksen ve işaret eşlemesini uygular. Rust adapter `direction_axis` JSONL operation'ı üzerinden yöntemi çağırır; Java oracle aynı enum metodunu doğrudan kullanır.

Senaryolar X/Y/Z eksenlerinin pozitif ve negatif AxisDirection çiftlerini kapsar. Beklenen yön snapshot'ları Java oracle çıktısından alındı.

Java math capture kimliği `a0fcce8574ed7e7504a909bf5d44782eb6188d8f866dbe5eb10e4a567cb0d772`, birleşik capture kimliği `e90cc09edae2a718af616d6ac887e78417d48304d0f8185161e41d82d8e17d85` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca axis-direction constructor sonucunu kapsar; kalan Direction bulma ve dönüşüm akışları ayrı davranış sözleşmeleridir.
