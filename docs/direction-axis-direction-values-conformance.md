# P01.c — Direction axis-direction-values referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.AxisDirection.getStep()` ve `opposite()` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:558-573` incelendi. Enum değerleri sabit signed step ve ad alanı taşır; `opposite()` iki enum değeri arasında doğrudan geçiş yapar.

## Davranış sözleşmesi

- `POSITIVE.getStep()` `1`, `NEGATIVE.getStep()` `-1` döndürür.
- `POSITIVE.getName()` ve `NEGATIVE.getName()` sırasıyla `Towards positive` ve `Towards negative` metinleridir.
- `opposite()` pozitif değeri negatife, negatif değeri pozitife çevirir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_direction_values"` ve `direction` (`positive` veya `negative`) alanlarını taşır. Çıktı `step`, `name` ve karşıt değerin `opposite` adını içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Axis yön eşlemeleri, predicate'ler, plane, random seçim, coordinate choose overload'ları ve Direction yönlerinin diğer sorguları bu dilime dahil değildir.


## Client-parity doğrulaması

`src/direction.rs` içindeki `AxisDirection.get_step` ve `opposite`, Java enum step değerlerini, adlarını ve karşıtlık ilişkisini uygular. Rust adapter `direction_axis_direction_values` JSONL operation'ı ile bu alanları dışarı verir; Java oracle aynı enum yöntemlerini doğrudan çağırır.

Senaryolar POSITIVE ve NEGATIVE değerlerinin step, ad ve karşıt sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `3e211af4291f51617cbb425f3c78bb3f4269726871ff3187b972beab0700d53f`, birleşik capture kimliği `ce65510770b37fb2ffa769c1f83b93c1ca651ebc4699a41192b45ba78541fbe7` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca AxisDirection değer yollarını kapsar; eksen yön eşlemeleri, predicate, plane ve coordinate seçimleri ayrı davranış sözleşmeleridir.
