# P16.g — Element döndürmesi — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik kutu, eksen, açı ve
origin pinned JAR'daki `CuboidRotation` ile `FaceBakery.rotateVertexBy`'a ve
üretim kütüphanesindeki `voxel_client::render::rotation` modülüne verilir;
dönüşüm matrisinin on altı elemanı ve dört döndürülmüş köşe float bit düzeyinde
karşılaştırılır.

Eksen hizalı kutu olmayan her blok buradan geçer: merdiven, meşale, ray, çit.

## Java kaynak incelemesi

`SingleAxisRotation`, sıfır açıda hiç trigonometriye girmeden birim matris
döndürür. Aksi halde açıyı radyana çevirip eksen etrafında döndürme matrisi
kurar. Dereceden radyana çevrim katsayısı **önce `float`'a daraltılır**, sonra
çarpılır; `double` içinde çarpıp sonra daraltmak son biti değiştirir.

Kritik ayrıntı kosinüsün nasıl elde edildiğidir. Referansın matematik
kütüphanesi kosinüsü doğrudan hesaplamaz; sinüsten Pythagoras kimliğiyle
büyüklüğünü alır ve işaretini açının çeyreğinden belirler. Bunun gözlenebilir
sonucu şudur: çeyrek turda kosinüs **negatif sıfırdır**, sıfır değil. Kosinüsü
doğrudan hesaplayan bir implementasyon burada pozitif sıfır üretir ve matrisin
o elemanı ayrışır.

`rescale` istendiğinde ve matris birim değilken, her eksen birim vektörü
dönüştürülür, en uzun bileşeninin tersi ölçek faktörü olur ve temel kolonları
bu faktörlerle çarpılır. Döndürülmüş bir elementin başladığı blok kenarına
değmeye devam etmesini sağlayan şey budur. Birim matris durumunda rescale
atlanır, yani sıfır açı hiç dokunulmadan kalır.

`rotateVertexBy` köşeyi origin'e taşır, dönüştürür ve geri taşır. Origin,
element ayrıştırıcısında `0.0625` ile ölçeklenerek blok birimine çevrilmiştir.

## Girdiler

`model/element-rotation` 69 satır taşır. Üç eksenin her biri için on bir açı
(`0`, `±22.5`, `±45`, `90`, `180`, `270`, `360`, `30`, `-7.5`) ve rescale açık
ile kapalı. Bunlardan `0` birim kısa yolunu, `90` negatif sıfırı, `180` ve `360`
işaret kombinasyonlarını, `30` ile `-7.5` ise gerçek modellerde bulunmayan fakat
dilbilgisinin izin verdiği açıları sürer. Ayrıca üç satır merkezde olmayan bir
origin ve kısmi bir kutu kullanır; merdiven ve meşalenin gerçek şekli budur.

## Sonuçlar

Java ve Rust her satırda aynı on altı matris elemanını ve aynı dört köşeyi,
float bit deseni düzeyinde verdi. Doğrulanan ayırt edici değerler:

| Girdi | Gözlenen |
| --- | --- |
| `y` ekseni, `22.5°` | `m00 = cos(22.5°)`, `m02 = -sin(22.5°)` |
| `y` ekseni, `22.5°`, rescale | `m00 = 1.0`, `m02 = -tan(22.5°)` |
| `z` ekseni, `90°` | `m00` **negatif sıfır** |
| herhangi bir eksen, `0°` | birim matris, rescale atlanmış |

Koşu: `python3 tools/behavior/run.py capture --family model` ardından
`compare --family model --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Açık kapsam

`EulerXYZRotation` (`x`/`y`/`z` alanlarıyla verilen üç eksenli döndürme) bu
kaydın dışındadır; referansın matris kütüphanesindeki `rotationZYX` sırası ve
onun kendi kosinüs türetmesi ayrıca sabitlenmelidir. Model durumu dönüşümü
(blockstate varyantının `x`/`y` döndürmesi), UV döndürme çeyrekleri,
`calculateFacing` ile normalden yön seçimi ve `recalculateWinding` köşe
yeniden sıralaması da açıktır.
