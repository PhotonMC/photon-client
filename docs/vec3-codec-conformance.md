# P01.b — Vec3 CODEC sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **12 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.CODEC` sabitinin üç double öğeli listeyi Vec3'e çevirmesi ve Vec3'ü x, y, z sırasındaki listeye kodlamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:19-23` içindeki codec tanımı ve `util/Util.java:965-973` içindeki sabit boyut kontrolü incelendi. Oracle, `JsonOps` üzerinde Java `Vec3.CODEC` çağrısını doğrudan yapar; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde JSON metninin sayı türlerini ayrıştıran genel `Codec.DOUBLE` davranışı ölçülmez. Adapter, `JsonArray` içine raw f64 bitlerinden oluşturulmuş Java `JsonPrimitive` değerleri koyar. Rust üretim API'si JSON veya `serde_json` almaz; sabit liste sözleşmesini f64 slice ve sabit boyutlu dizi ile taşır.

## Davranış sözleşmesi

- Decode girdisi tam üç double öğesi içerirse bunlar x, y ve z olarak aynı sırayla yeni bir Vec3'e aktarılır.
- Decode girdisi üçten kısa veya boşsa `Input is not a list of 3 elements` hatası döner ve partial Vec3 bulunmaz.
- Decode girdisi üçten uzunsa aynı hata döner; partial değer ilk üç öğeden oluşturulan Vec3'tür. Sonraki öğeler partial değere girmez.
- Encode sonucu x, y ve z bileşenlerini aynı sıradaki üç double listesi olarak taşır.
- Signed zero, Infinity ve NaN bileşenleri codec'in listeye aktardığı ham f64 değerlerinde korunur. Karşılaştırma tolerans veya NaN normalizasyonu kullanmaz.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Decode tam liste | 5 | Temel, pozitif sıfır, signed zero, subnormal/büyük sonlu ve özel değerler; Vec3 x/y/z ile türetilmiş uzunluk bitleri |
| Decode yanlış boyut | 3 | Boş, kısa ve uzun listeler; hata metni, uzun listede ilk üç öğenin partial Vec3'ü |
| Encode | 4 | Temel, sıfır, signed zero ve özel değerler; x/y/z sırasındaki üç raw f64 bit string'i |

Girdiler `family: "math"`, `operation: "vec3_codec"` ve `mode: "decode" | "encode"` kullanır. Decode isteğinde `values` 16 haneli küçük harfli f64 bit string listesi; encode isteğinde `x`, `y`, `z` aynı biçimdeki bileşenlerdir. Başarılı decode `result.value` içinde Vec3 snapshot'ı, başarılı encode üç öğeli bit listesi taşır. Hatalı decode `result.outcome`, `result.error` ve uzun liste için `result.partial` alanlarını taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::codec_decode`, tam üç öğeyi kabul eder; yanlış boyutta gerçek uzunluğu ve yalnızca üç veya daha fazla öğe için ilk üç öğeden oluşan partial değeri `Vec3CodecError::WrongLength` içinde taşır. `Vec3::codec_encode`, bileşenleri `[x, y, z]` sırasıyla döndürür. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java codec sonucunu doğrudan `DataResult` üzerinden gözlemler.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.CODEC` sabitinin sabit üç öğeli liste sözleşmesini `Vec3::codec_decode` ve `Vec3::codec_encode` ile karşılar. Java oracle codec'i `JsonOps` üzerinde doğrudan çağırır; Rust adaptörü aynı raw f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Vec3 CODEC için 8 decode ve 4 encode olmak üzere 12 özgün senaryo hazırlandı.
- Tam üç öğeli decode x/y/z sırasını ve signed zero, subnormal, Infinity ve NaN bitlerini korudu; boş/kısa listeler partial üretmeden hata verdi; uzun liste ilk üç öğeden partial Vec3 üretti.
- Rust debug ve release adaptörleri güncel 2299 senaryoluk math ailesinde **2299/2299** eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `Vec3.CODEC` içindeki üç öğeli sabit boyut dönüşümünü, x/y/z encode sırasını, raw f64 bitlerini ve `Util.fixedSize` kısa/uzun liste sonucunu kapsar. Genel JSON sayı türü ayrıştırması, başka codec'lerin bileşimi ve oyun paketlerindeki kullanım yolları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
