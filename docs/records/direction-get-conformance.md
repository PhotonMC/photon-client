# P01.c — Direction get referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.get(AxisDirection, Axis)` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:369-377` incelendi. Yöntem altı Direction enum değerini tarayarak verilen axis-direction çiftine karşılık gelen yönü döndürür.

## Davranış sözleşmesi

- X pozitif/negatif EAST/WEST, Y pozitif/negatif UP/DOWN, Z pozitif/negatif SOUTH/NORTH sonuçlarını verir.
- Java çağrısındaki parametre sırası AxisDirection sonra Axis'tir.
- Enumların tüm geçerli çiftleri test edilir; geçersiz bir enum kombinasyonu üretilemez.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_get"`, `axis` (`x`, `y` veya `z`) ve `direction` (`positive` veya `negative`) alanlarını taşır. Çıktı seçilen yön snapshot'ını içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

`fromAxisAndDirection`, isim çözümleme, codec yolları ve diğer Direction yardımcıları ayrı davranış sözleşmeleridir.


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki `Direction::get`, Java `Direction.get(AxisDirection, Axis)` lookup'ını parametre sırası korunarak uygular. Rust adapter `direction_get` JSONL operation'ı üzerinden yön snapshot'ını dışarı verir; Java oracle static lookup metodunu doğrudan çağırır.

Altı axis-direction çifti Java oracle çıktısıyla karşılaştırılır.

Java math capture kimliği `96c733cc1ba2e170149c44d674090b3cde045749fc67c8bcf8a2fb09049a7155`, birleşik capture kimliği `0c710be232812e0f45b5412691acde56c06561cf701020ad71949bf4e31a3f0a` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.get` lookup yolunu kapsar; `fromAxisAndDirection` ve diğer Direction yolları ayrı davranış sözleşmeleridir.
