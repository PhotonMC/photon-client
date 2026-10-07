# T60 — `sounds.json` çözümü ve ağırlıklı ses seçimi — [client-parity]

Bu kayıt oyunun ses tanımlarını (`sounds.json`) ve bir olaydan ağırlıkla ses seçimini kapsar. Kanıt sınıfı `client-parity`'dir: gerçek `SoundManager`'ın ayrıştırıcısı ve kayıt işlemi (`SoundEventRegistrationSerializer`, `Preparations.handleRegistration`) ile gerçek `WeighedSoundEvents.getSound` aynı girdide ve aynı rastgele tohumla Rust ile karşılaştırılır. Ses çıkışı ve çözme bu kaydın dışındadır.

## Java kaynak incelemesi

- `sounds.json` varlık dizininde (`minecraft/sounds.json`) durur; `SoundManager.prepare` bir dosyayı bütün olarak ayrıştırır: içinde geçersiz bir şey olan dosyadan **hiçbir olay kaydedilmez** (günlüğe uyarı yazılır). Geçerli dosyada her olay `handleRegistration` ile eklenir: ad ya yeni, ya `replace: true`; var olan olaya eklenir.
- Girdi: yalnız ad (dize) ya da nesne (`name`, `type` `file`/`event`, `volume`, `pitch` > 0, `weight` > 0, `preload`, `stream`, `attenuation_distance` varsayılan 16). Dosya girdisinin `sounds/<ad>.ogg` dosyası yoksa o girdi atlanır. Olay girdisinin ağırlığı, gösterdiği olayın **o anki** toplam ağırlığıdır (olay yoksa 0).
- Seçim: `nextInt(toplamAğırlık)`, sonra ağırlıklar çıkarılarak ilk negatif; boş ya da sıfır ağırlıklı olay `EMPTY_SOUND` (`minecraft:empty`) verir. Olay girdisi seçilince gösterdiği olaydan (aynı rastgele kaynakla) seçer; ses ve perde çarpılır (`1 · sarılan · girdi`), akış ikisinden biri, ön yükleme ve sönüm sarılanın.
- Gson'un ilkel dönüşümleri: `subtitle`/`name`/`type` sayı ve boole değerlerini metin sayar; `replace`/`preload`/`stream` için boole olmayan ilkel `Boolean.parseBoolean`dır (sayı → yanlış); `volume`/`pitch`/`weight`/`attenuation_distance` sayı değilse hata.

## Kapsam

`sound/registry` 21 satır: küçük dosyalar (düz adlar, ağırlıklar, alanlar, olay girdileri ve iç içe olaylar, olmayan olay, olmayan dosyalar, boş olay, `replace`, ad alanlı adlar, ondalık ağırlık, büyük ses) ve her bir hata türü (sıfır ses, negatif perde, sıfır ağırlık, geçersiz tür, adsız, nesne olmayan, dizi olmayan, sayı alt yazı, sayı boole); bir **gerçek** satır: oyunun gerçek `sounds.json`'u (1968 olay) ve gerçek dosya listesiyle, tüm kayıt dökümü ve altı olaydan 40'ar seçim.

Rust: `voxel-game::sounds` (`SoundRegistry`, `Sound`, `SampledFloat`).

## Sapmalar ve açık kalanlar

- Birden çok paketin/ad alanının dosya yığını ve `replace`'in paketler arası etkisi tek dosyayla sınanır; kaynak paketi desteği ayrı iştir.
- Gson'un yinelenen anahtar hatası ve geçersiz olay adı (`IdentifierException`) için satır yok; gerçek dosyada yoktur.
- `Component.translatable` çözümü (alt yazı metni) yok; yalnız anahtar saklanır.
