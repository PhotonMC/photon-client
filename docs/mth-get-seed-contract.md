# P01.a — `Mth.getSeed(int, int, int)` referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki koordinat overload'unun signed long sonucunu ölçer. Girdi koordinatları JSON'da i32 aralığında tutulur; sonuç signed i64 ondalık biçimde taşınır.

## Kaynak ve sınır

Java kaynağı `util/Mth.java:327-335` incelendi. `getSeed(Vec3i)` overload'u yalnızca getter forwarding yaptığı için bu dilimde doğrudan koordinat overload'u çağrılır. Positional random factory'nin aynı aritmetiği kullanması [random fork sözleşmesinde](random-fork-contract.md) ayrıca gözlenir.

## Davranış sözleşmesi

- `x * 3129871`, iki i32 operandın Java signed overflow davranışıyla hesaplanır ve sonra long'a yükseltilir.
- Bu terim, `z * 116129781L` ve `y` ile kaynak sırasını koruyan long XOR zincirine girer.
- Ara seed, Java long genişliğinde iki çarpım ve bir toplama ile sarılır.
- Sonuç arithmetic signed `>> 16` ile üretilir; unsigned kaydırma veya saturating arithmetic kullanılmaz.
- Koordinatların sırası `x, y, z` olarak korunur. Zero, signed sınırlar ve taşma duyarlı girdiler ayrı referans senaryolarıdır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "mth_get_seed"`, `x`, `y` ve `z` alanlarını içerir. Java oracle doğrudan `Mth.getSeed(x, y, z)` çağırır; beklenen değerler Rust çıktısından türetilmez.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Mevcut capture'lar değiştirilmez; kaynak, girdi veya ortam değiştiğinde yeni referans kimliği oluşur.

## Kapsam dışı

`getSeed(Vec3i)` için ayrı bir Rust Vec3i overload'u, farklı random source türleri ve istemcinin dünya/oyuncu seed tüketim sırası bu dilime dahil değildir.
