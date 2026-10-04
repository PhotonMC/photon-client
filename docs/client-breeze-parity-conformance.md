# T36e — Breeze — [client-parity]

Bu kayıt, breeze'in üç modelinin (gövde, rüzgâr, gözler) ve istemci durumunun (poz değişince ve her tikte
başlayan animasyon durumları) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir:
modeller gerçek `BreezeModel`'e, tik gerçek `Breeze` varlığına karşı karşılaştırılır. Katmanların boru
hattı ayarları (kaydırma, aydınlatma) oracle ile karşılaştırılmaz. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `BreezeModel.setupAnim`: altı animasyon (boşta, ateş, kayma, geri kayma, soluma, uzun zıplama), durumları
  başlamışsa. Üç katman (gövde ve çubuklar, rüzgâr, gözler) aynı parça hiyerarşisini taşır (ayıklanan parçalar boş
  kalır), bu yüzden aynı animasyonlar üçüne de uygulanır.
- `Breeze.onSyncedDataUpdated(DATA_POSE)` (istemci): ateş, boşta, soluma ve zıplama durdurulur (kayma ve geri kayma
  durdurulmaz); sonra ateş, soluma ya da kayma pozu kendi durumunu (durmuşsa) başlatır.
- `Breeze.tick` (istemci, tikin başında): uzun zıplama pozunda zıplama durumu başlar; boşta durumu her tik
  (durmuşsa) başlar; poz kayma değilken kayma çalışıyorsa geri kayma başlar ve kayma durur.
- Çizim: gövde `entityTranslucent`; rüzgâr `BREEZE_WIND` (doku `u` ekseninde yaşın fiftide biri kadar kayar ve tekrarlar,
  dünya ışığıyla aydınlanır, yüz aydınlatması ve bindirme yok); gözler `ENTITY_TRANSLUCENT_EMISSIVE` (yüz aydınlatması var,
  dünya ışığı yok).
- Paketlenmiş görüntü tekrarlanmaz: kaydırılan dokunun sınırı aşan yüzleri sınırdan kesilir, ötesi dokunun
  başından çizilir (bir birim testle denetlenir). Entity gölgelendiricisi iki bant içi işaret kazandı: sıfır normal
  "yüz aydınlatması yok", negatif ışık "dünya ışığı yok"; böylece yeni boru hattı gerekmedi.

## Kapsam

`entity/breeze-model` 33 satır (üç katmanda: durumsuz, boşta, her animasyon tek başına erken ve geç, boşta ile ateş,
kayma ile geri kayma); `living-entity-animations`'a 8 satır (boşta, ateş, soluma ve ateşe geçiş, kayma, kayıp durma,
yeniden kayma, uzun zıplama, ateşin yeniden gönderilmesi). Entity gölgelendiricisinin derlendiği bir birim testle denetlenir.

## Sapmalar (açık)

- Gözlerin derinliğe yazmaması (`DepthStencilState(.., false)`) ve rüzgârın `sortOnUpload` sıralaması uygulanmaz.
  Dokuların yolları renderer sabitinden elle yazıldı. Gölge, ses, parçacıklar, rüzgâr topu yok. Canlı gözlem
  yapılmadı; yeni gölgelendirici dalları yalnız derlenme düzeyinde denetlendi.
