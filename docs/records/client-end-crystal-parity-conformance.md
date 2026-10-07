# T37h3 — Ender kristali ve kristal ışını — [client-parity]

Bu kayıt, end kristalinin modelini (yüzen cam küpler), ışın geometrisini (`submitCrystalBeams`) ve istemci durumunu
(verileri, ejderhanın en yakın kristali seçmesi) Java 26.2 ile aynı sonucu vermesi için kapsar. Kanıt sınıfı `client-parity`'dir:
model gerçek `EndCrystalModel`'e, ışın gerçek `submitCrystalBeams`'e, durum gerçek `EndCrystal`/`EnderDragon` tikine karşı
karşılaştırılır. Boru hattı ayarları ve ışığın çizimi oracle ile karşılaştırılmaz. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `EndCrystalModel.setupAnim`: taban `showsBottom` ise görünür; dış cam `getY(yaş)·16/2` kadar yükselir; üç iç içe küp,
  eğik bir eksen (`π/3`, `(sin45, 0, sin45)`) etrafında `yaş·3` derecelik dönüşle çevrilir (`ModelPart.rotateBy`: Euler
  açılarına matris yoluyla). `getY`: `(h² + h)·0.4 − 1.4`, `h = sin(yaş·0.2)/2 + 0.5`.
- `EndCrystalRenderer`: 2 kat ölçek, `(0, −0.5, 0)`; ışın hedefi varsa (`BeamTarget`, bloğun merkezi) kristalin konumundan
  hedefe; `submitCrystalBeams(−dx, −dy + getY(yaş), −dz, yaş, …)` hedefin konumundan çizilir. `EndCrystal.time` rastgele
  başlar (tohumsuz) ve her tik bir artar.
- `EnderDragonRenderer.submitCrystalBeams`: `(0, 2, 0)`, yaw (`−atan2(dz, dx) − π/2`) ve pitch (`−atan2(yatayUzunluk, dy) − π/2`)
  dönüşleri; sekiz kenarlı tüp: her kenar için dört köşe, başlangıçta (0.2 ölçekli, siyah, alfa 255) ve sonda (tam ölçekli,
  beyaz), doku `v` boyunca `uzunluk/32 − yaş·0.01` tekrar eder. Boru hattı (`END_CRYSTAL_BEAM`): kesme 0.1, bindirme yok,
  iki yüz, **yüz başına aydınlatma yok** (köşenin normaliyle tek bir yan); Rust'ta bu, köşe gölgesinin `−1` olmasıyla
  işaretlenir. Siyah köşe rengi, gölgelendiricide negatif renkle işaretlenir (sıfır "renk yok" demektir). Paketlenmiş
  görüntü tekrarlanmaz: uzun dörtgenler her tekrarda kesilir, rengi kesim noktalarında siyahtan beyaza aradeğerlenir.
- `EnderDragon.checkCrystals`: (ölmüyorsa) tuttuğu kristal dünyadan gittiyse bırakılır, durursa her 10 tikte can bir artar
  (istemcide de); onda bir tikte, kutusu kendi kutusunun 32 blok genişletilmişiyle kesişen kristallerden varlıklar arası
  uzaklığı en küçük olan seçilir. Rastgele kaynak tohumsuzdur; oracle yalnız son tikte (seçilmiş olmalı) karşılaştırır.
  Ayrıca ölürken ejderhanın konumu artık yaklaşmaz (`aiStep` ölüm dalında hiçbir şey yapmaz).
- Kristal ve ejderha ışını aynı geometriyi kullanır; ejderhanınki ejderhanın konumundan, dönmeden çizilir (`beamOffset`:
  kristal, camının yüksekliyle, ejderhadan).

## Kapsam

`entity/end-crystal-model` 18 satır (dokuz yaşta, tabanlı ve tabansız); `entity/crystal-beam` 13 satır (eksenler, çapraz, kısa,
uzun, tik, ışık, kaydırılmış poz; köşelerin konum, doku, renk, normal ve sırasını Java ile karşılaştırır);
`living-entity-animations`'a 6 satır (kristal varsayılanı, ışın ve taban, ışını yitirme; ejderha kristal bulur, uzağı bulmaz,
erişimin kıyısında bulur).

## Sapmalar (açık)

- Işık ve sis, ışının gölgelendirici dalları (siyah işareti, tek yan) oracle'sız; kristalin canlı karesi alındı, ışının görünür
  bir karesi alınamadı. `EndCrystal` tehlike/yangın, vurma ve ses yok. Ejderha canının bir artması zamanlaması rastgele
  seçime bağlı olduğundan karşılaştırılmaz. Doku yolları elle yazıldı.
