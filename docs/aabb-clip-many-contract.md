# P01.d — AABB iterable clip ve BlockHitResult sözleşmesi

Bu dilim, Java `AABB.clip(Iterable<AABB>, Vec3, Vec3, BlockPos)` overloadunu ve başarılı dönüşte üretilen `BlockHitResult` alanlarını kapsar. Dokuz özgün Java davranış senaryosu hazırlandı; Rust karşılığı [doğrulandı](aabb-clip-many-parity.md).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:314-330` içindeki iterable clip akışı, `:333-388` içindeki yön ve yüz taraması, `world/phys/BlockHitResult.java:6-65` içindeki hit alanları incelendi. `MathOracle` aday kutuları Java `AABB` constructor'ı ile oluşturup doğrudan statik clip çağrısını gözlemler; sonucu yeniden hesaplamaz.

## Davranış sözleşmesi

- Segment farkı `to - from` olarak hesaplanır ve ortak en yakın ölçek başlangıçta `1.0`'dır. Aday kutular liste sırasıyla işlenir.
- Her aday `BlockPos` koordinatlarıyla `AABB.move(BlockPos)` üzerinden taşınır. Bu taşıma, aday kutunun dünya konumunu değiştirir; dönen `BlockHitResult` içindeki `blockPos` ise verilen `pos` değeridir.
- Hareket farkı `1e-7` eşiğini aşan eksenlerde, ışının gidiş yönüne göre karşı yüz taranır. X, sonra Y, sonra Z sırası korunur; aynı ölçeğe sahip sonraki yüz strict `<` koşulu nedeniyle önceki yönü değiştirmez.
- Yüz adayı `s = (face - fromA) / deltaA` ile hesaplanır. Yalnızca `0 < s < önceki_ölçek` ve diğer iki koordinat `1e-7` toleransında yüz aralığındaysa ortak ölçek ve yön güncellenir.
- Hiçbir aday geçerli değilse sonuç `null`'dır. Hit varsa konum `from + s * (to - from)` ile üretilir ve `BlockHitResult` türü `BLOCK`, `inside` ve `worldBorderHit` alanları `false` olur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Batı yüzü | 1 | Pozitif X hareketiyle unit box girişi |
| Doğu yüzü ve taşıma | 1 | Negatif X hareketi, offset `BlockPos` |
| Dikey ve derinlik yüzleri | 1 | Aşağı/yukarı/kuzey/güney doğrultularını zincir halinde sorgulama |
| Köşe önceliği | 1 | Eşit ölçekli diagonal hit; X yüzü önce kalır |
| Uzak sonra yakın | 1 | Liste sırasına rağmen en yakın kutunun seçilmesi |
| Yakın sonra uzak | 1 | Sonraki uzak kutunun yakın sonucu değiştirmemesi |
| Boş iterable | 1 | Hiç aday kutu olmaması |
| Taşınmış miss | 1 | `BlockPos` taşımasından sonra segment dışında kalan kutu |
| İçeriden ve eşik altı | 1 | Java clip'in başlangıç içi ve küçük delta davranışı |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içindeki `op: "clip_many"`, f64 bit alanlı `boxes`, `from`/`to` noktaları ve decimal i32 `pos` nesnesini taşır. Hit sonucu konum, yön, BlockPos, tür ve sabit hit bayraklarıyla; miss sonucu yalnızca `hit: false` ile snapshot'lanır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin math referans kimliği `577f55bea348590f76cc3685cd87aadd677ba8f4649d2acfa055ee6138565773` oldu. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
