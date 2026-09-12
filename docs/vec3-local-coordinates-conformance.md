# P01.b — Vec3 yerel koordinat yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.applyLocalCoordinatesToRotation` statik yardımcı metodu ile `addLocalCoordinates` instance metodunun yerel yönü dünya koordinatlarına çevirmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:327-349` ve trigonometrik bağımlılık olarak `util/Mth.java:29-55` incelendi. `addLocalCoordinates`, önce `this.rotation()` sonucunu üretip aynı statik yardımcıyı çağırır. Oracle doğrudan Java metodlarını çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `applyLocalCoordinatesToRotation`, pitch/yaw Vec2'sinden ileri, yukarı ve sol f64 Vec3 tabanlarını Java `Mth` sin/cos lookup sonuçlarıyla kurar.
- Açı dönüşümü ve taban vektörlerinin çarpımları Java'daki f32 ara değerleriyle yapılır; taban bileşenleri Vec3 constructor'ına geçerken f64'e genişletilir.
- Sol taban, ileri ve yukarı vektörlerinin cross sonucunun `-1.0` ile ölçeklenmesiyle elde edilir.
- Yerel yönün x/y/z bileşenleri sırasıyla sol, yukarı ve ileri tabanlarıyla f64 çarpılıp Java kaynak sırasıyla toplanır.
- `addLocalCoordinates`, mevcut vektörün `rotation()` sonucunu kullanır ve verilen yönü aynı dönüşümden geçirir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `apply-basic` | Sıfır pitch/yaw ve sonlu yön | Vec3 snapshot'ı |
| `apply-combined` | Birleşik pitch/yaw ve negatif/pozitif yön bileşenleri | Vec3 snapshot'ı |
| `apply-axis-turns` | 90/180 derece eksen dönüşleri | Vec3 snapshot'ı |
| `apply-signed-zero` | Signed-zero açı ve subnormal yön bileşenleri | Vec3 snapshot'ı |
| `add-axis` | Z ekseni yönelimi üzerinden instance çağrısı | Vec3 snapshot'ı |
| `add-combined` | Negatif/non-unit mevcut vektör ve birleşik yerel yön | Vec3 snapshot'ı |

Girdiler `family: "math"`, `operation: "vec3_local_coordinates"` ve `kind` alanını kullanır. `apply` için f32 bit dizgeli `rotation` Vec2'si ve f64 bit dizgeli `direction` Vec3'ü; `add` için f64 bit dizgeli `first` ve `direction` Vec3'leri kullanılır. Sonuç `value` içinde f64 `x`, `y`, `z`, `length` ve `length_squared` bitlerini taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::apply_local_coordinates_to_rotation` aynı f32 trigonometrik ara yolu ve f64 toplama sırasını taşır. `Vec3::add_local_coordinates`, `rotation()` sonucunu bu yardımcıya verir. `examples/behavior_adapter.rs` statik ve instance yollarını doğrudan bu API'lerden sorgular; `tests/behavior/java/MathOracle.java` aynı yolları Java sınıfına bağlar.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.applyLocalCoordinatesToRotation` ve `addLocalCoordinates` yardımcılarını aynı f32 trigonometrik taban ve f64 bileşen toplama sırasıyla sunar. Java oracle doğrudan Java metodlarını çağırır; Rust adaptörü aynı sonuçları üretim API'sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Dört `applyLocalCoordinatesToRotation` ve iki `addLocalCoordinates` senaryosunda Vec3 snapshot bileşenleri, uzunluk ve uzunluk karesi f64 bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört statik dönüşüm, iki instance forwarding ve bu girdilerdeki f32/f64 özel bit sonuçlarıyla sınırlıdır. Kamera, oyuncu hareketi ve diğer yerel yön tüketicileri sonraki istemci dilimleridir.
