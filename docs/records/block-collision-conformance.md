# P17.b — Blok çarpışma şekilleri — [client-parity] kayıt

**Tarih:** 2026-09-27 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: 1196 bloğun bütün durumlarının çarpışma
şekli, her parity koşusunda referansın kendi `getCollisionShape`'iyle
karşılaştırılıyor.

## Neyin yerini aldı

`ClientBlockStateRegistry` şimdiye kadar elle yazılmış, kasıtlı olarak küçük bir
alt küme taşıyordu: hava, taş, birkaç levha ve merdiven. Alt kümenin dışındaki
her durum havaya düşüyordu; yani bir çit, kapı ya da duvar hareketi hiç
durdurmuyordu. Alt küme kaldırıldı, yerini üretilmiş tam tablo aldı.

## Ölçüm

Boş bağlamla (`CollisionContext.empty()`) 32366 durumun şekli **326 farklı
şekle** iniyor. Şekiller kutularına göre anahtarlanıyor, çünkü referans her
durum için yeni bir nesne veriyor.

Tablonun yalnızca başlangıç noktası olduğu bloklar ölçülüp işaretlendi:

- **Konuma bağlı** (ölçülerek): `bamboo`, `pointed_dripstone`,
  `sulfur_spike`. Rastgele bir ofsetle konumlarına göre kayıyorlar.
- **Dinamik** (`hasDynamicShape`): yukarıdakiler, shulker kutuları,
  `moving_piston`, `powder_snow`, `scaffolding`. Toz kar ve iskele şekli
  soran varlığa bakıyor; shulker ve piston blok varlığını okuyor. Oyuncu
  hareketi toz karın şeklini düşme mesafesinden kendisi kuruyor
  (`docs/records/local-player-tick-conformance.md`).
- **Bağlama bağlı ama oyuncu için boş**: su ve lav. `LiquidBlock` şekli
  yalnızca sıvı üstünde durabilen bir canlı ya da `alwaysCollideWithFluid`
  için dolu; oyuncu ikisi de değil, yani tablonun boş şekli oyuncu için doğru.

## Doğrulanan şey

Adaptör kutuları tablodan değil, Rust'ın tablodan **yeniden kurduğu**
şekillerden yayıyor. Böylece karşılaştırma iki şeyi birden sınıyor: tablonun
kendisini ve kutulardan şekil kurarken birleştirmenin referansla aynı kutuları
vermesini. 326 şeklin hepsi eşleşti.

Tek tam blok kutusu, kutudan kurulmuş bir şekil yerine referansın kendi tam
blok şekli olarak kuruluyor; çarpışma yürüyüşü o şekli kimlikle tanıyıp kısa
yol alıyor.

## Sonuç

`parity_passed`, 3905 senaryo / 13 aile. 202 birim testi. Mevcut dünya
çarpışma senaryolarının hepsi yeni tabloyla da geçti.

Şekil tabloları artık yok: çarpışma ve çizgi şekilleri blok sınıflarının
kodundan hesaplanıyor (`crates/voxel-world/src/block/shape_rules/`) ve
oracle vakaları bu hesaplanan şekilleri karşılaştırıyor. Konuma göre kayan
yalnızca çiçek, bambu, sarkıt ve mangrov filizi (referansta da
`shape.move(getOffset(pos))` yapan sınıflar); çimen gibi blokların şekli
kaymaz, yalnız çizimi kayar. Çarpışma şekli dünyada bloğun kendi konumundaki kaymayla
verilir (`collision_shape_at`); konumsuz sorulduğunda orijindeki kaymayla.

## Açık kapsam

Konuma bağlı üç bloğun ofseti ve dinamik bloklar (toz kar, iskele, shulker,
piston) henüz tablodaki değerle çalışıyor. İskele ve toz kar oyuncunun
konumuna ve inip inmediğine bakıyor; bunlar hareket bileşimi pinlenirken
gerçek oyuncuyla sürülecek.
