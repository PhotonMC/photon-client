# P01.d — AABB deflate üç eksenli overload sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.deflate(double, double, double)` overload'unun eksen başına miktarları `inflate` yoluna ters işaretle aktarmasını içerir. Scalar `deflate(double)` ve diğer AABB işlemleri ayrı senaryolarla izlenir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:286-292` incelendi. `MathOracle`, üç f64 miktarı doğrudan Java `AABB.deflate` overload'una verir; beklenen uçlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Üç eksenli overload her miktarın işaretini Java unary negation ile çevirip `inflate` üç eksenli yoluna aktarır.
- Pozitif miktar alt ve üst endpoint'leri içeri taşır; negatif miktar kutuyu o eksende genişletir.
- Miktarlar eksen başına bağımsızdır. Sonuç Java `AABB` kurucusunun endpoint normalizasyonundan geçer.
- Signed zero, NaN ve sonsuzluk değerleri Java double negation ve aritmetik davranışıyla korunur; Rust çağrı yolu debug ve release'te aynı bitleri üretir.
- Ardışık adımlar önceki AABB sonucunu sonraki `deflate_xyz` veya scalar `deflate` çağrısına verir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-mixed` | Karışık işaretli üç eksen miktarı | Eksen başına endpoint sonucu |
| `vector-reversed` | Boyutu aşan pozitif miktarlar | Constructor normalizasyonu |
| `signed-zero` | Signed zero miktarları | Endpoint bitleri |
| `chain` | Üç eksenli ve scalar ardışık adımları | Sıralı AABB snapshot'ları |
| `special-values` | NaN ve sonsuzluk | Java özel değer bitleri |

Girdiler `family: "math"`, `operation: "aabb"` ve `steps` içinde `op: "deflate_xyz"` kullanır. Miktarlar f64 bit string'leridir; her adım sonrası kutu endpoint'leri `events` içinde taşınır.

## Rust karşılığı

`crates/photon-core/src/aabb.rs` içindeki `Aabb::deflate_xyz`, Java'nın ters işaretli `inflate` yönlendirmesini uygular. Java unary negation'ın NaN işaretini release optimizasyonlarından bağımsız korumak için bit düzeyinde yardımcı kullanılır. `examples/behavior_adapter.rs` bu metodu JSONL `deflate_xyz` adımına bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/aabb.rs`, Java `AABB.deflate(double, double, double)` overload'unun ters işaretli `inflate` davranışını `Aabb::deflate_xyz` ile uygular. Java oracle üç miktarlı overload'u doğrudan çağırır; Rust adapter aynı girdiyi üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda karışık eksen miktarları, constructor normalizasyonu, signed zero, zincir ve NaN/sonsuzluk yolları karşılaştırıldı.
- NaN senaryosunda release derlemesinin Java unary negation bitlerini koruduğu ayrıca doğrulandı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik AABB step dizisi ve bunların endpoint snapshot'larıyla sınırlıdır. Üç eksenli overload'un doğrudan davranışı doğrulandı; istemci hareketi ve dünya collider toplama sonraki P01/P09 dilimleridir.
