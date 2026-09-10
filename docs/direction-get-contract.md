# P01.c — Direction get referans sözleşmesi

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
