# P16.q — UV kilidi — [client-parity] kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir. [P16.l](model-state-conformance.md) bu dilimi
açık bırakmıştı; engel kaldırıldı ve nasıl kaldırıldığı burada yazılı.

## Engel neydi, nasıl aşıldı

Kilitli yüz dönüşümleri referansta kuaterniyondan matris kurularak,
bileştirilerek ve tersi alınarak hesaplanıyor. O değerler **tam değil**:
tabloda `0.99999994` ve `-1.0000001` gibi girdiler var. Yeniden hesaplamak
matris kütüphanesinin `rotate`, `mul` ve `invertAffine` formüllerinin harfiyen
aynısını gerektiriyordu ve o formüller yerel referans kaynağında yok.

Çözüm formülleri tahmin etmek değil, **tabloyu referanstan veri olarak
almak** oldu. Referansın kendisi de bunu bir tablo olarak tutuyor: açılışta bir
kez hesaplayıp küp simetri grubunun her elemanı için önbelleğe alıyor.

Ölçüm: altmış dört `(x, y, z)` çeyrek tur birleşimi sürüldüğünde **tam olarak
yirmi dört** farklı tablo çıktı — grubun eleman sayısı — ve her dönüşüm
matrisine tek bir tablo düştü. Dönüşüm matrisleri tam (yalnızca `0`, `±1`)
olduğu için tablo onlarla anahtarlanabiliyor. Tablonun tamamı yalnızca on
farklı kayan nokta değeri taşıyor, bu yüzden `crates/voxel-data/src/model/uv_lock.rs`
değerleri bir kez listeleyip matrisleri indekslerle yazıyor.

Bu tablo **her parity koşusunda baştan sona karşılaştırılıyor**: 24 dönüşüm ×
6 yüz × 2 matris × 16 float. Yani gömülü veri, doğrulanmamış bir sabit değil.

## Java kaynak incelemesi

`FaceBakery.bakeVertex` yalnızca `inverseFaceTransformation(facing)` okuyor.
Koordinat önce yüzün merkezine taşınıyor (`u - 0.5`), matris onu bir **konum**
olarak dönüştürüyor, sonra geri taşınıyor.

Kritik ayrıntı `MatrixUtil.isIdentity` kısayolu. Birim matris ise ham
koordinat dokunulmadan geçiyor; değilse dönüşümden geçiyor. Karşılaştırma
JOML'un `determineProperties`'i üzerinden **değere göre** yapılıyor, bite göre
değil. İkisi de önemli:

- Negatif sıfır sıfır sayılıyor. Tablolarda `-0.0` girdileri var ve referans
  o matrisleri yine de birim kabul ediyor. Bunu bit karşılaştırmasıyla yazmak
  ilk denememde `x0-y180-north` satırında parity'yi düşürdü.
- Birden kıl payı farklı bir değer birim **sayılmıyor**. Yani o tablolar
  dönüştürme yoluna giriyor ve koordinatı bir tık kaydırıyor.

## Girdiler

`model/model-state` artık kilitli satırlar da taşıyor: dört × dört × dört
çeyrek tur birleşiminin hepsi, her biri altı yüzün ileri ve ters matrisiyle.

`model/uv-lock` 696 satırda uygulamayı sürüyor: on altı `(x, y)` birleşimi ×
altı yüz × yedi örnek koordinat (köşeler, merkez, çeyrekler ve yüzün dışına
düşen iki nokta), artı `z` ekseninin tek başına sürüldüğü altı satır ve
kilidin kapalı olduğu altı satır.

## Sonuçlar

Java ve Rust her matrisi ve her dönüştürülmüş koordinatı float bit düzeyinde
aynı verdi. `identity` bayrağı da her satırda eşleşti.

Koşu: `python3 tools/behavior/run.py capture` ardından
`compare --adapter target/debug/examples/behavior_adapter` → `parity_passed`,
3900 senaryo / 12 aile.

## Çizime bağlanması

`build_mesh` artık yüz başına bir UV dönüşümü alıyor ve sonda bunu seçilen
varyanttan besliyor. `minecraft:oak_stairs` `facing=north` durumunda
`y=270, uvlock=true` istiyor; eskiden uyarı basılıyordu, artık uygulanıyor.

## Açık kapsam

`BlockMath.getFaceTransformation` kilitli yolda normali dönüştürüp en yakın
yönü seçiyor. O adım tablonun **içinde** kaldığı için ayrıca sabitlenmedi;
tablo değiştiğinde (yeni bir sürümde) yeniden üretilmesi gerekir:
`tools/generate/uv_lock.py` `model/model-state` vakasının `uvlock-`
satırlarını `transformation`'a göre tekilleştirip indeksli tabloyu yazar.
