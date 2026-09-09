# P01.d — AABB koordinat clip overloadu sözleşmesi

Bu dilim, Java `AABB.clip(double, double, double, double, double, double, Vec3, Vec3)` statik overloadunu kapsar. Altı özgün Java davranış senaryosu hazırlandı ve Rust karşılığı [doğrulandı](aabb-clip-coordinates-parity.md).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:298-311` içindeki koordinat overloadu ve `getDirection`/`clipPoint` akışı incelendi. `MathOracle` her adımda statik Java çağrısını doğrudan yapar; kesişim noktası veya yön oracle içinde yeniden hesaplanmaz.

## Davranış sözleşmesi

- Altı koordinat, mevcut kutu kurucusundan geçirilmez; statik clip taramasına ham endpoint olarak aktarılır. Ters endpoint sırası bu nedenle çağrı girdisinin parçasıdır.
- Rust API bu altı ham koordinatı normalleştirmeden iki `Vec3` (`min`, `max`) ve segmentin iki `Vec3` (`from`, `to`) parametresiyle taşır.
- `from` ve `to` farkı eksen bazında hesaplanır. Her eksen yalnızca farkı `+1e-7` değerinden büyük veya `-1e-7` değerinden küçükse aday yüz taramasına girer.
- Aday yüzün ölçeği `s = (yüz - fromA) / deltaA` olarak bulunur. Aday yalnızca `0 < s < önceki_en_yakın_s` koşulunda ve diğer iki koordinat yüz aralığının `1e-7` toleransı içindeyse kabul edilir.
- Kabul edilen aday en küçük ölçeği ve buna karşılık gelen kesişim noktasını saklar. Hiç aday kabul edilmezse sonuç yoktur; aksi halde `from + s * (to - from)` döner.
- Sonuç yalnızca `hit` ve hit durumunda üç f64 bit alanlı `point` snapshot'ı taşır. Yön bilgisi bu overloadun dönüşünde bulunmaz.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Pozitif x taraması | 1 | Batıdan doğuya unit box ışını |
| Negatif x taraması | 1 | Doğudan batıya unit box ışını |
| Çok eksenli tarama | 1 | Ters olmayan geniş kutu ve diagonal ışın |
| Paralel miss | 1 | Kutu dışındaki sabit y ile x doğrultusu |
| Ham ters endpoint | 1 | Min/max sırası ters koordinat kutusu |
| Eşik ve özel değerler | 1 | Eşik altı hareket, sıfır hareket, NaN ve sonsuzluk |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "clip_coords"` bulunur; altı endpoint f64 bit dizgeleri, `from` ve `to` üç bileşenli f64 bit snapshot'larıdır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimin math referans kimliği `30579b0337c9cbef64bfe390fc1b1e3c6fb20fd2e3c1d1f1ebb1626d7805d642` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
