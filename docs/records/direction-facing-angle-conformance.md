# P01.c — Direction facing-angle referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.isFacingAngle(float)` yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:395-400` ve `util/Mth.java:50-55` incelendi. Yöntem derece cinsinden f32 açıyı, Java'nın `(float) (Math.PI / 180.0)` katsayısıyla f32 radyana çevirir; ardından Mth'in quantized sinüs/kosinüs tablosuyla yatay bakış vektörü kurar.

## Davranış sözleşmesi

- `radians`, derece katsayısı ve açı çarpımından f32 olarak elde edilir; lookup çağrıları bu değeri double'a genişletir.
- `dx = -Mth.sin(radians)` ve `dz = Mth.cos(radians)` olarak hesaplanır.
- Yönün integer normalindeki X/Z bileşenleri bu f32 değerlerle çarpılır ve toplam strict `> 0.0F` ile doğru/yanlış olarak sonuçlanır.
- Dikey yönlerde X ve Z normal bileşenleri sıfır olduğundan açı ne olursa olsun sonuç false kalır. Eşitlik, signed zero ve NaN strict pozitiflik testini geçmez.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_is_facing_angle"`, `name` yön adı ve IEEE-754 f32 `angle` alanını 8 hexadecimal bit olarak taşır. Çıktı `value` boolean'ıdır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Direction seçimi, entity bakış sıralaması, random yön, codec'ler ve hareket vektörünün diğer yardımcıları bu dilime dahil değildir.


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki `is_facing_angle`, Java `Direction.isFacingAngle(float)` derece/radyan f32 dönüşümünü, Mth sinüs/kosinüs lookup yolunu, yön normalinin X/Z ürünlerini ve strict pozitiflik kararını uygular. Rust adapter `direction_is_facing_angle` JSONL operation'ı ile boolean sonucu dışarı verir; Java oracle aynı Direction yöntemini doğrudan çağırır.

Senaryolar yatay ve dikey yönleri, cardinal/diagonal/negatif açıları, signed zero, subnormal, sonsuzluk, NaN ve f32 sınırlarını kapsar. Beklenen boolean sonuçlar Java oracle çıktısından alınır.

Java math capture kimliği `10123a3164046b82429b3881a351d93b17b5e7d74bb0381b4376a9580c3c16ab`, birleşik capture kimliği `c3e71daf3bde4157f96bdcd8e7e99da4aa758649aa8ee7ac499bb7de6aa348e0` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca facing-angle sonucunu kapsar; diğer Direction bulma, dönüştürme ve entity bakış akışları ayrı davranış sözleşmeleridir.
