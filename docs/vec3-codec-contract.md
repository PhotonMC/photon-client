# P01.b — Vec3 CODEC sözleşmesi

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
