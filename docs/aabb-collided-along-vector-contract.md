# P01.d — AABB collidedAlongVector sözleşmesi

Bu dilim, Java `AABB.collidedAlongVector(Vec3, List<AABB>)` davranışını kapsar. Sekiz Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](aabb-collided-along-vector-parity.md).

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

Bu dilimin math referans kimliği `e8e76ccc62aed58e30c8ea04f5862929a6644eaba7132217f369f558775216a5` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](aabb-collided-along-vector-parity.md).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
