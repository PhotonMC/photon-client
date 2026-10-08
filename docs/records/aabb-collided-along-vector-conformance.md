# P01.d — AABB collidedAlongVector sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java `AABB.collidedAlongVector(Vec3, List<AABB>)` davranışını kapsar. Sekiz Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](#client-parity-doğrulaması).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:401-414` içindeki `collidedAlongVector` akışı ile kullandığı `getCenter`, boyut, `inflate`, `contains` ve `clip` yolları incelendi. `MathOracle` her sorguda Java metodunu doğrudan çağırır; beklenen boolean sonucu oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- Başlangıç noktası hareketli kutunun merkezidir; bitiş noktası bu merkeze verilen Vec3 vektörünün eklenmesiyle bulunur.
- Her aday kutu, hareketli kutunun her eksendeki boyutunun yarısından `1e-7` çıkarılarak eksen başına inflate edilir. Inflate sonucu Java AABB kurucusunun endpoint normalizasyonundan geçer.
- Inflate edilmiş kutu bitiş veya başlangıç noktasını içeriyorsa çarpışma vardır. Contains alt sınırı dahil, üst sınırı hariç tutar.
- Bu iki nokta da içeride değilse, aynı inflate edilmiş kutu ile merkezler arasındaki `clip` çağrısı denenir; geçerli bir segment kesişimi çarpışma sayılır.
- Aday kutular verilen liste sırasıyla incelenir; ilk pozitif sonuçta sorgu biter. Boş liste veya bütün adayların negatif olması `false` üretir.
- NaN, sonsuzluk, signed zero, çok küçük boyut ve epsilon sınırları Java double aritmetiği ile karşılaştırmalarının doğal sonucudur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Boş liste | 1 | Aday kutu olmadan merkez hareketi |
| Başlangıç noktası | 1 | Inflate edilmiş adayın başlangıç merkezini içermesi |
| Bitiş noktası | 1 | Inflate edilmiş adayın bitiş merkezini içermesi |
| Segment geçişi | 1 | İki merkez noktası dışarıdayken inflate edilmiş kutuyu geçme |
| Kaçırma | 1 | Segmentin diğer eksende adaydan ayrılması |
| Çoklu aday | 1 | İlk aday kaçırır, sonraki aday çarpışır |
| Epsilon ve özel değerler | 1 | Sıfır hareketi, epsilon üstü hareket ve NaN vektörü |
| Çok küçük hareketli kutu | 1 | Negatif inflate miktarıyla daralan aday |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "collided_along_vector"`, f64 bit dizgeli `vector` ve f64 bit dizgeli AABB nesnelerinden oluşan `boxes` dizisi bulunur. Her adım `hit` boolean snapshot'ı ekler.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimin math referans kimliği `e8e76ccc62aed58e30c8ea04f5862929a6644eaba7132217f369f558775216a5` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](#client-parity-doğrulaması).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/photon-core/src/aabb.rs`, Java `AABB.collidedAlongVector(Vec3, List<AABB>)` yolunu hareketli kutunun merkez segmenti, aday kutuların boyut tabanlı inflate edilmesi, contains kontrolleri ve `clip` sonucu üzerinden uygular. `examples/behavior_adapter.rs` çoklu AABB listesini JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 456 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **456/456** math senaryosunda eşleşti.
- Sekiz yeni senaryoda boş liste, başlangıç/bitiş contains, segment geçişi, kaçırma, çoklu aday, epsilon ve NaN davranışları ile negatif inflate miktarı karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e8e76ccc62aed58e30c8ea04f5862929a6644eaba7132217f369f558775216a5`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-doqiofma` (math debug), `compare-jpkfv0nx` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `AABB.collidedAlongVector`, mevcut `AABB.clip(Vec3, Vec3)` ve JSONL list adaptörüyle sınırlıdır. Çoklu AABB clip sonraki dilimdir.
