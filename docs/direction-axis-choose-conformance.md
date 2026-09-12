# P01.c — Direction axis-choose referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Axis.choose` int, double ve boolean overload'larını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:539-543` incelendi. Her overload seçilen enum eksenine karşılık gelen tek x, y veya z bileşenini döndürür.

## Davranış sözleşmesi

- X ekseni üç overload'da x girdisini seçer.
- Y ekseni üç overload'da y girdisini seçer.
- Z ekseni üç overload'da z girdisini seçer.
- int sonucu signed i32 olarak, double sonucu ham IEEE-754 bitleriyle, boolean sonucu doğrudan korunur.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_choose"` ve `axis` alanını taşır. `int_x/y/z` signed i32 dizgeleri, `double_x/y/z` f64 bit dizgeleri ve `bool_x/y/z` boolean alanlarıdır. Çıktı seçilen `int`, `double` ve `boolean` değerlerini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Axis predicate'leri, yön eşlemeleri, plane, random seçim ve Direction yönlerinin diğer sorguları bu dilime dahil değildir.


## Client-parity doğrulaması

`src/direction.rs` içindeki `Axis.choose_i32`, `choose_f64` ve `choose_bool`, Java `Direction.Axis.choose` overload'larının x/y/z seçim davranışını uygular. Rust adapter `direction_axis_choose` JSONL operation'ı ile üç sonucu dışarı verir; Java oracle overload'ları doğrudan çağırır.

Senaryolar üç eksendeki ayırt edici değerleri, signed i32 sınırlarını ve özel f64 bitlerini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `f5118835b6978b7f3daf585aaa981852456e02841b516150b3a8fba3685b1747`, birleşik capture kimliği `7afa4dfb0877ea063c2fafe11985de74d797d985562f727d6bb15b286c892367` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Axis.choose overload'larını kapsar; predicate, yön eşlemeleri, plane ve diğer Direction sorguları ayrı davranış sözleşmeleridir.
