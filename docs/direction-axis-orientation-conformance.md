# P01.c — Direction axis-orientation referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Axis.isVertical()` ve `isHorizontal()` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:498-504` incelendi. Predicate'ler yalnızca enum kimliğini karşılaştırır; koordinat veya yön normalinden hesap yapmaz.

## Davranış sözleşmesi

- `Y.isVertical()` true, `X` ve `Z` false döndürür.
- `X.isHorizontal()` ve `Z.isHorizontal()` true, `Y` false döndürür.
- Her geçerli eksen için iki predicate birbirinin tamamlayıcısıdır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_orientation"` ve `axis` (`x`, `y` veya `z`) alanlarını taşır. Çıktı `vertical` ve `horizontal` boolean alanlarını içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Eksenin yön seçimi, `getPlane`, random seçim, coordinate choose overload'ları ve Direction yönlerinin diğer sorguları bu dilime dahil değildir.


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `Axis.is_vertical` ve `is_horizontal`, Java enum predicate'lerinin üç eksendeki sonuçlarını uygular. Rust adapter `direction_axis_orientation` JSONL operation'ı ile iki boolean sonucu dışarı verir; Java oracle aynı enum yöntemlerini doğrudan çağırır.

Senaryolar X, Y ve Z eksenlerinin dikey/yatay sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `74b808f59d73d90fc04e7c4eb3f5989ed2ecb60babf1889877bb88d5e7766bc3`, birleşik capture kimliği `021b9773c00ad5713ff63f4ea7be38710c58e8afccc9b129d2974803f6fa7877` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca axis-orientation predicate'lerini kapsar; eksen yönü, plane, random ve coordinate seçimleri ayrı davranış sözleşmeleridir.
