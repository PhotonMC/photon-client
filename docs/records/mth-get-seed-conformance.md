# P01.a — `Mth.getSeed(int, int, int)` referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki koordinat overload'unun signed long sonucunu ölçer. Girdi koordinatları JSON'da i32 aralığında tutulur; sonuç signed i64 ondalık biçimde taşınır.

## Kaynak ve sınır

Java kaynağı `util/Mth.java:327-335` incelendi. `getSeed(Vec3i)` overload'u yalnızca getter forwarding yaptığı için bu dilimde doğrudan koordinat overload'u çağrılır. Positional random factory'nin aynı aritmetiği kullanması [random fork sözleşmesinde](random-fork-conformance.md) ayrıca gözlenir.

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


## Client-parity doğrulaması

`crates/photon-core/src/math.rs` içindeki `mth_get_seed`, Java `Mth.getSeed(int, int, int)` koordinat overload'unun i32 ve i64 taşma sırasını uygular. `crates/photon-core/src/random.rs` positional factory'si de aynı yardımcıyı çağırır; ayrı bir seed formülü tutulmaz.

Java oracle `Mth.getSeed(x, y, z)` çağrısını doğrudan yürütür. Girdilerde sıfır, işaretli i32 sınırları, `x * 3129871` i32 taşması ve büyük karışık koordinatlar bulunur. Beklenen sonuçlar Java oracle çıktısından alındı.

Java referans capture kimliği:

`19ecd56114bfeca6651a19ed8170f87981da3d7b7e9998f3aa0e880ac3a311a8`

Yeni birleşik capture kimliği:

`d29deef1874c152ce4860c236af9875602b0f92e24aa57b9ded67ac5d4c379cb`

Math adaptörüyle karşılaştırma geçti:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Birleşik capture, debug ve release adaptörleriyle de karşılaştırıldı:

```bash
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/release/examples/behavior_adapter
```

Bu kanıt koordinat overload'u ile sınırlıdır. `getSeed(Vec3i)` forwarding'i ve world/positional random tüketim sırası daha geniş veri akışlarının parçasıdır.
