# P01.b — Vec3 equals/hashCode sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `Vec3.equals(Object)` ve `Vec3.hashCode()` metodlarının üç f64 bileşen üzerindeki Java karşılaştırma ve hash kurallarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:196-221` içindeki `equals` ve `hashCode` gövdeleri incelendi. Oracle doğrudan Java metodlarını çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `equals`, her bileşeni `Double.compare` ile karşılaştırır ve üç karşılaştırmanın da sıfır olmasını ister.
- Java karşılaştırmasında `+0.0` ile `-0.0` farklıdır; bütün NaN payload'ları canonical double karşılaştırması nedeniyle eşit kabul edilir.
- `hashCode`, her bileşen için `Double.doubleToLongBits` sonucunu yüksek ve düşük 32 bitlerin XOR'una indirger, sonra bileşenleri 31 tabanında birleştirir.
- Hash hesabı Java `int` aritmetiğiyle taşar; NaN payload'ları aynı canonical hash değerine sahiptir.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Eşit sonlu bileşenler | Eşitlik ve temel hash formülü |
| `different-component` | Tek bileşeni farklı vektörler | Eşitsizlik ve hash ayrımı |
| `signed-zero` | `-0.0` ile `+0.0` | Signed zero karşılaştırması ve hash'i |
| `nan-payloads` | Farklı NaN payload'ları | NaN canonicalization |
| `infinities` | Aynı pozitif/negatif Infinity bileşenleri | Sonsuzluk eşitliği |
| `special-mismatch` | Infinity, NaN ve signed zero farkları | Özel değerlerin birlikte karşılaştırılması |

Girdiler `family: "math"`, `operation: "vec3_equality"` ve f64 bit dizgeli `first`/`second` Vec3 nesnelerinden oluşur. Sonuç eşitlik boolean'ı ile iki Java int hash değerini decimal string olarak taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::equals_java` ve `Vec3::hash_code_java`, Java'nın canonical double karşılaştırmasını ve wrapping int hash aritmetiğini uygular. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` aynı Java metodlarını çağırır.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.equals(Object)` ve `Vec3.hashCode()` davranışlarını karşılar. Java oracle, karşılaştırmayı ve iki hash sonucunu doğrudan Java metodlarından alır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu değerler, farklı bileşen, signed zero, farklı NaN payload'ları, Infinity ve özel değerlerin birlikte karşılaştırılması ölçüldü.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 eşleştirmesinde `equals` boolean'ı ile `hashCode` decimal string sonuçlarının Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. `Vec3.toString()` ve diğer nesne biçimlendirme davranışları ayrı dilimlerdir.
