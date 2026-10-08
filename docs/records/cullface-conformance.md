# P16.r — Yüz gizleme (cullface) — karışık kanıt kaydı

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

İki kanıt sınıfı:

- `client-parity`: bir yüzün `cullface` yönünün varyant döndürmesiyle nereye
  düştüğü, ve bunun dayandığı en yakın yön araması.
- **Basitleştirilmiş**: komşunun o yüzü gerçekten kapatıp kapatmadığı kararı.
  Referansta `VoxelShape` örtüşmesine ve blok durumuna bakıyor; sonda "dolu
  bir blok komşusu kapatır" diyor.

## Java kaynak incelemesi

Bir yüz `cullface` ile arkasına saklandığı yönü adlandırır. Model döndüğünde o
yön de döner, bu yüzden `UnbakedCuboidGeometry.bake` komşuyu sormadan önce
`Direction.rotate(modelState.transformation().getMatrix(), cullForDirection)`
uyguluyor.

`Direction.rotate` normali dönüşümden geçirip `getApproximateNearest` çağırıyor.
Bu arama `findClosestDirection`'dan **farklıdır** ve farkı önemlidir:

- Eşik sıfırdan değil, Java'nın en küçük pozitif float'ından (`1.4e-45`,
  ki bu subnormal bir değerdir ve Rust'ın `f32::MIN_POSITIVE`'i değildir)
  başlıyor.
- Hiçbir aday eşiği geçemezse sonuç `null` değil, aramanın başlangıç değeri
  olan **north** oluyor.
- Sonlu olma denetimi yok.

Ölçülen sonuçlar bunu doğruluyor: sıfır vektörü, eşiğin altında kalan
`1e-46`, negatif sıfırlar ve `NaN` hep `north` veriyor; `-inf, 0, 0` ise
`west` veriyor çünkü batı normaliyle iç çarpımı `+inf`.

Eşitlikte önceki yön kazanıyor: `(1, 1, 1)` için sıra `down, up, north,
south, west, east` olduğundan `up` çıkıyor.

## Girdiler

`model/nearest-direction` on yedi satır: altı eksen, sıfır, hepsi negatif,
köşegen, üçlü eşitlik, eşiğin hemen üstü ve altı, negatif sıfır, `NaN`, iki
sonsuz ve karışık bir vektör.

`model/cull-rotation` 384 satır: dört × dört × dört çeyrek tur birleşiminin
hepsi × altı yüz.

## Sonuçlar

Java ve Rust her satırda aynı yönü verdi.

Koşu: `python3 tools/behavior/run.py capture` ardından
`compare --adapter target/debug/examples/behavior_adapter` → `parity_passed`,
3902 senaryo / 12 aile.

## Çizime bağlanması

`build_mesh` artık bir örtülme yordamı alıyor ve `cullface` taşıyan bir yüzü,
döndürülmüş yönündeki komşu kapatıyorsa atıyor.

Sonda `PHOTON_GRID=N` ile bloğu bir küp ızgara hâlinde tekrarlıyor — yüz
gizlemenin iş yaptığı en küçük sahne. Sayılar tam çıkıyor:

| Izgara | Küp | Çizilen yüz | Beklenen dış yüz |
| --- | --- | --- | --- |
| 3 | 27 | 54 | `6 × 3²` = 54 |
| 4 | 64 | 96 | `6 × 4²` = 96 |

Yani bütün iç yüzler düşüyor, tek bir fazlası kalmıyor.

## Açık kapsam

Gerçek karar `Block.shouldRenderFace`: komşunun `getFaceOcclusionShape`'i tam
bir blok ise gizle, boş ise gizleme, arada ise iki şeklin farkını alıp
önbellekli bir örtüşme testi yap. Ayrıca `state.skipRendering(neighborState,
direction)` aynı türden iki bloğun (cam, buz) ortak yüzünü gizliyor.

Üçü de blok durumuna ve kayıt defterine bağlı. Sonda yerine "ızgara içinde bir
komşu varsa kapalıdır" diyor; bu dolu küpler için doğru sonucu veriyor ama
merdiven, cam ya da yaprak için vermez.
