# T64a — Blok sesleri ve oyuncunun hareket sesleri — [client-parity] (tablo) · [server-conformance] (tetikler)

Bu kayıt iki şeyi ayrı sınıflarla tutar. **Tablo** (`client-parity`): her bloğun ses türü (`SoundType`: ses, perde ve kırma/adım/koyma/vurma/düşme sesleri) gerçek kayıt defterinden alınır ve her koşuda baştan sona karşılaştırılır. **Tetikler**: oyuncunun hareket sesleri (adım, yüzme, su sıçraması, iniş, kabarcık sütununa girme, hasar ve ölüm) gerçek `LocalPlayer`'a karşı karşılaştırılır (`client-parity`, `movement/local-player-ticks` satırlarının `sounds` alanı: olay, kaynak, ses, perde, yer; bit bit). Vuruş, kırma ve koyma sesleri Java kaynağından yazıldı ve birim testli/canlı gözlemlidir (`server-conformance`).

## Java kaynak incelemesi

- `SoundType`: yalnız `DecoratedPotBlock` durumdan duruma türünü değiştirir (çatlak küp); gerisi bloğun varsayılan durumunun türüdür.
- Adım: `Entity.applyMovementEmissionAndPlaySound` — yürünen yatay (tırmanırken tüm) uzaklık `·0,6` ile `moveDist`'e eklenir, `moveDist > nextStep` ve ayak altı hava değilse, blok üstünde durulurken (yerde, tırmanırken ya da eğilip yatayda sabitken, yüzmüyorken) adım sesi çalınır ve `nextStep = (int)moveDist + 1` olur; suda yüzme sesi. `Player.getMovementEmission`: uçarken ya da yerde eğilmişken ses yok. `Player.playStepSound`: suda `waterSwimSound` + bastırılmış adım; üstteki blok adım sesini geçiriyorsa onun sesi (birleşik adım sesli bloklarda `0,15` ve bastırılmış `0,05·0,8`); adım sesi `ses·0,15`, perde türün perdesi.
- Vuruş: `MultiPlayerGameMode.continueDestroyBlock`, `destroyTicks % 4 == 0` iken `(ses+1)/8`, perde `·0,5`, blok merkezinde. Kırma: `LevelEvent 2001` (yerel oyuncu için `Block.playerWillDestroy` aynı olayı yerelde tetikler, başkaları için sunucu paketi): `(ses+1)/2`, perde `·0,8`, `BLOCKS`, blok merkezi. Koyma: `BlockItem.place` aynı ölçeklerle koyma sesi.

## Yapılan

- `block_sounds` tablosu (`tools/generate/block_sounds.py`, `photon-data::block_sound`; 127 tür, 1196 blok, 8 durum istisnası) ve oracle'ı (`sound/block-sounds`).
- `LocalPlayer`: `moveDist`/`nextStep`, adım ve yüzme sesleri (`take_sounds`); dünya: `push_sound`/`take_sounds`, `play_hit_sound`, `play_place_sound`, `apply_level_event(2001)`; `ClientboundLevelEventPacket` çözümü (yalnız 2001 işlenir); oyun modu vuruş/kırma/koyma tetikleri; uygulama sesleri motora verir.

## Sapmalar ve açık kalanlar

- Oracle'da `SoundManager` sessiz bir sahtedir ve çalınan sesleri kaydeder. Rastgele olanlar karşılaştırılmaz: ortam sesleri (su altı, mağara) elenir, yüzme ve sıçrama sesinin perdesi (varlığın tohumsuz rastgelesi) `0`a eşitlenir, bal bloğunda kayarken beşte bir şansla çalan ses elenir (inişteki kesin ses karşılaştırılır). Yerel `Entity.random` bir tohumdan başlar, Java'daki gibi rastgele değil.
- Binekteki oyuncunun sesleri karşılaştırılmaz: araç inince düşüşü yolcuya aktarır (`propagateFallToPassengers`) ve binek tiklerinde bu aktarım yok. Slime bloğu (düşme hasarı sıfır) ses çıkarmaz. Çiftlik toprağı ve kaplumbağa yumurtası yalnız sunucuda bozulur. Dürtülmüş ucu ve yatak yolları kodda var, sahnede yok. Darbe bağlamı (`currentImpulse`) yok.
- Ametist tınısı, kapı/sandık, yağmur, mağara/su altı döngüleri ve **müzik** (`MusicManager`) yok; bunlar [T64d](../tasks/T64d-ses-tetikleri-ortam-muzik.md)dedir.
- `LevelEvent`'in 2001 dışındaki olayları (parçacıklar, kapı sesleri vb.) yok sayılır ([PLAN-M8](../tasks/README.md)).
- Hasar sesi: hasar türünün `effects` değerinden (`sound/damage-effects` tablosu, oracle'dan); pakette yazılı hasar türü desteklenmez. Perde rastgele olduğundan `0`a eşitlenir.
- Eşya kullanım sesleri yok. Adım sesinde `isSilent` yok.
