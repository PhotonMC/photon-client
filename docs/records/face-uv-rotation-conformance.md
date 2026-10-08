# P16.i — Yüz UV döndürmesi — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün UV dikdörtgeni ve derece değeri
pinned JAR'daki `Quadrant.parseJson`, `CuboidFace.getU` ve `getV` yoluna ve
üretim kütüphanesindeki `photon::render::geometry` modülüne verilir; dört
köşenin doku koordinatları float bit düzeyinde karşılaştırılır.

## Java kaynak incelemesi

Bir yüzün `uv` dikdörtgeninin köşeleri köşe yuvalarına sabit bir sırayla
düşer: ilk iki yuva düşük `u`'yu, birinci ve dördüncü yuva düşük `v`'yi alır.
Yani yuva sırası `(min,min)`, `(min,max)`, `(max,max)`, `(max,min)`'dir.
Değerler on altıya bölünerek döndürülür.

`rotation` alanı yuvaları kaydırır: `rotateVertexIndex` her köşe indeksine
çeyreğin kaymasını ekler ve dörde göre sarar.

`parseJson` dereceyi önce pozitif aralığa sarar. Bu yüzden `-90` ile `270`,
`450` ile `90`, `-360` ile `0` aynı çeyreği verir. Doksanın katı olmayan bir
değer yuvarlanmaz, **reddedilir**.

## Girdiler

`model/face-uv-rotation` 80 satır taşır: beş UV dikdörtgeni (tam blok, kısmi,
`u` ekseninde çevrilmiş, iki eksende çevrilmiş, kesirli) çarpı on altı derece
değeri. Dereceler dört kabul edilen çeyreği, onlara sarılan pozitif ve negatif
değerleri (`360`, `450`, `720`, `-90`, `-180`, `-270`, `-360`) ve reddedilmesi
gereken beş değeri (`45`, `30`, `1`, `-45`, `100`) içerir.

Çevrilmiş dikdörtgenler gerçek modellerde dokuyu aynalamak için kullanılır ve
`min > max` olduğunda da aynı yuva sırasının işlediğini sabitler.

## Sonuçlar

Java ve Rust her satırda aynı kabul/ret kararını, aynı kaymayı ve aynı dört
koordinat çiftini verdi.

Koşu: `python3 tools/behavior/run.py capture --family model` ardından
`compare --family model --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Çizim yoluna bağlandı

[Dokulu küp](textured-cube-record.md) artık köşe doku koordinatlarını elle
yazılmış bir eşleme yerine bu tablodan alıyor. Değişiklik davranış korur:
yeniden çizilen kare önceki kareyle bit-bit aynı çıktı.

## Açık kapsam

Sprite'a bağlı UV paketlemesi açıktır: buradaki koordinatlar birim kareye
göredir, atlas üzerindeki gerçek konum sprite yerleşimine bağlıdır. Model
durumu dönüşümünün yüze uyguladığı ters dönüşüm de bu kaydın dışındadır.
