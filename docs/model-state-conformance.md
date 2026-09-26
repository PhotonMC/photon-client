# P16.l — Blockstate varyant döndürmesi — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı `x` ve `y` çeyrek turları pinned
JAR'daki `Quadrant.fromXYAngles` → `BlockModelRotation` yoluna ve üretim
kütüphanesindeki `voxel_client::render::rotation::model_state_transform`
fonksiyonuna verilir; dönüşüm matrisi ve altı yüzün dönüşümleri float bit
düzeyinde karşılaştırılır.

Bir blockstate varyantı modelini çeyrek turlarla döndürebilir. `bakeQuad` bu
dönüşümü her köşeye uygular, yani merdivenin hangi yöne baktığı buradan gelir.

## Java kaynak incelemesi

İki açı önce çeyreğe çevrilir, sonra küp simetri grubunun iki üretecine
eşlenir ve `y` üreteci `x` üretecinin dışına sarılarak bileştirilir.

Gözlenen önemli bir ayrıntı: grubun matrisleri yalnızca sıfır ve artı/eksi bir
içerir. Yani bileşim tamdır ve trigonometri gerektirmez; `90` derecelik bir
dönüşün kosinüsünü hesaplayan bir implementasyonun taşıyacağı yuvarlama riski
bu yolda yoktur.

İkinci ayrıntı: varyantın `y: 90` değeri grubun **negatif** doksanlık Y
elemanına karşılık gelir. Üreteç matrisleri bu yüzden kaynaktan okunmuş
değerlerdir, işaret konvansiyonu tahmin edilmemiştir.

UV kilidi kapalıyken yüz dönüşümlerinin tamamı birim matristir.

## Girdiler

`model/model-state` 64 satır taşır: `x` ve `y` için sekiz açı
(`0`, `90`, `180`, `270`, `360`, `-90`, `-180`, `450`) çaprazlanır. Sarılan
değerler çeyrek eşlemesinin `positive_modulo` ile yapıldığını sabitler.

## Sonuçlar

Java ve Rust her satırda aynı dönüşüm matrisini ve aynı altı yüz dönüşümünü
verdi.

Koşu: `python3 tools/behavior/run.py capture --family model` ardından
`compare --family model --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Açık kapsam

**UV kilidi bu kaydın dışındadır.** Kilitli yüz dönüşümleri kuaterniyondan
kurulan bir tablo üzerinden geçer, normali dönüştürüp en yakın yönü bulur ve
ters dönüşümle bileştirir; o yol kendi dilimini hak eder ve buradaki tam sayı
güvencesini taşımaz.

Varyant seçiminin kendisi de açıktır: `blockstates/` dosyasından blok
durumuna göre varyant seçmek referansta blok registry'sine bağlıdır. Bu kayıt
yalnızca seçilen varyantın döndürmesinin ne ürettiğini sabitler. Ağırlıklı
varyant listeleri ve `multipart` koşulları da dışarıdadır.
