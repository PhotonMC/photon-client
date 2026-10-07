# P16.e — Cuboid yüz geometrisi — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik kutu ve yön, pinned
JAR'daki `FaceBakery.defaultFaceUV` ile `FaceInfo` köşe tablosuna ve üretim
kütüphanesindeki `voxel_client::render::geometry` modülüne verilir; dört köşe
konumu ve varsayılan UV dikdörtgeni float bit düzeyinde karşılaştırılır.

## Java kaynak incelemesi

Her blok quad'ı burada başlar. `FaceInfo`, altı yönün her biri için dört köşeyi
`MIN_X`/`MAX_Y` gibi seçicilerle sabit bir sırada tanımlar. Bu sıra quad'ın
sarım yönünü belirler; ters sarılmış bir yüz aynalanmaz, elenir. `bakeVertex`
köşeyi seçer seçmez on altıya böler, yani konumlar blok biriminde üretilir.

`defaultFaceUV`, model bir UV vermediğinde yüzün dokuda nereye düşeceğini
hesaplar. Her yön farklı bir eksen çifti okur ve altının dördü eksenlerden
birini `16 - değer` ile çevirir; blok dokusunun her yönden dik durmasını
sağlayan şey budur.

## Girdiler

`model/face-geometry` 48 satır taşır: sekiz kutu × altı yön. Kutular tam blok,
slab, içeri kaçık, ince dikey çubuk, bir eksende sıfır kalınlıklı dejenere
kutu, element dilbilgisinin izin verdiği blok dışına taşan kutu, kesirli
sınırlar ve sınırları ters çevrilmiş kutudur. Sonuncusu köşe tablosunun kendi
başına geçerlilik denetimi yapmadığını sabitler.

## Sonuçlar

Java ve Rust her satırda aynı dört köşeyi ve aynı UV dikdörtgenini, float bit
deseni düzeyinde verdi.

Koşu: `python3 tools/behavior/run.py capture --family model` ardından
`compare --family model --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Açık kapsam

Element döndürmesi (`rotation`, `origin`, `rescale`), model durumu dönüşümü,
UV döndürme çeyrekleri, `calculateFacing` ile normalden yön seçimi ve
`recalculateWinding` köşe yeniden sıralaması bu kaydın dışındadır. Sprite'a
bağlı UV paketlemesi de dışarıdadır: atlas üzerindeki gerçek koordinat
[atlas kaydına](atlas-stitcher-conformance.md) ve sprite içeriğine bağlıdır.
