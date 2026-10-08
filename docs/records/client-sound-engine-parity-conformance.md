# T61b — `SoundEngine` mantığı ve kanal parametreleri — [client-parity]

Bu kayıt ses motorunun kayıt tutmasını kapsar: hangi ses hangi kanalda çalıyor, kanala ne söyleniyor (perde, ses, sönüm, konum, döngü, göreli), ses nasıl gecikir, tiklenir, döngüye girer, duraklar, durdurulur ve bırakılır. Kanalın kendisi (sesi çıkaran şey) arayüzün arkasındadır ve sonraki görevdedir ([T63](../tasks/T63-ses-cikisi-cpal.md)). Kanıt sınıfı `client-parity`'dir: gerçek `SoundEngine` ve `ChannelAccess`, kanalı yerine çağrıları yazan bir sahteyle koşturulur; kanala giden çağrı dizisi Rust'ınkiyle karşılaştırılır.

## Java kaynak incelemesi

- `play`: `canPlaySound`, olay çözümü (`resolve`: bilinmeyen olay, `intentionally_empty`, boş ses → başlamaz), sönüm mesafesi `max(ses, 1)·attenuation_distance`, ses = `clamp(ses, 0, 1) · clamp(kategori·ana, 0, 1) · kazanç`, perde `clamp(perde, 0.5, 2)`, sıfır ses yalnız `canStartSilent` ya da müzikle başlar (sessiz), kanal havuzundan kanal alınır (akışlı sesler ayrı havuzdan; havuz doluysa başlamaz), kanala sırayla: perde, ses, doğrusal ya da kapalı sönüm, döngü (`döngü ∧ ¬akış ∧ gecikme ≤ 0`), konum, göreli; sonra tam tampon ya da akış (`döngü ∧ gecikme ≤ 0` ise döngülü), `play`.
- `tick`: sıradaki tikli sesler çalınır; tikli her ses `tick()` olur, durdu ise durdurulur, yoksa ses, perde ve konumu kanala yeniden verilir; kanalı bırakılmış sesler **en az 20 tik** sonra unutulur (elle döngülü olan, `gecikme` tik sonrasına kuyruğa girer); gecikmeli sesler zamanı gelince (`tick()` sonra `play`) çalınır; duraklatılmışken yalnız biten müzik unutulur. `ChannelAccess.scheduleTick`: akışlar beslenir, biten kanallar bırakılır (havuza döner).
- `stop` (örnek, ad ve/veya kaynak; ikisi de yoksa `stopAll`: bütün kanallar bırakılır, kuyruklar ve kazançlar silinir), `pauseAllExcept`, `resume`, `updateCategoryVolume` (kazanç sıkıştırılır, kaynağın ya da ana kategorinin sesleri yenilenir), `refreshCategoryVolume`, `isActive`.

## Kapsam

`sound/engine` 9 senaryo: her parametre türüyle çalma (sönümsüz, göreli, döngülü, akışlı, olay girdili, aşırı ses ve perde), reddedilenler (bilinmeyen/boş/kasıtlı boş olay, çalınamaz, sıfır ses, sessiz başlama, müzik, ana kaynak, negatif ses), kaydırıcılar ve kazançlar, havuzun tükenmesi, örnek/ad/kaynak ile durdurma ve `stopAll`, 20 tikin sonunda unutulma, gecikme ve elle döngü, tikli sesler (kuyruğa alınan, gecikmeli), duraklatma ve devam.
- Rust: `photon-game::sound_engine` (`SoundEngine`, `SoundInstance`, `Channel`/`ChannelSource` arayüzleri, `SoundVolumes`); `SoundInstance::at` bir `SoundPacket`'ten `SimpleSoundInstance`'ı kurar.
- Oracle: `Channel`, `Library`, `SoundBuffer`, `Listener`, `ListenerTransform`, `DeviceList`, `DeviceTracker` test içinde yazılan sahtelerdir (`tests/behavior/java/com/mojang/blaze3d/audio/`; imzalı paket aynı pakette karışamadığından hepsi gölgelenir); kanal havuzu sayımı sahtede yazılmıştır (gerçeğiyle aynı sayma). Referans kanalları kimlik özeti sırasıyla gezdiğinden çağrılar kanal kanal gruplanıp kanal sırasına konur.

## Sapmalar ve açık kalanlar

- Kanal havuzunun gerçek `Library.CountingChannelPool`'u kullanılmadı (sahte aynı sayar); aygıt değişimi ve `Listener` (dinleyici dönüşümü) yok.
- `SoundEventListener`'lar (alt yazılar) ve `EntityBoundSoundInstance` (entity'yi izleyen ses: tikinde dünya erişimi ister) uygulanmadı; ilk çıkışla birlikte (T63/T64).
- Engine mantığı eşzamanlı çağrılar yapar; gerçek çıkışta kanal komutlarının ayrı bir iş parçacığına verilmesi T63'ün konusudur.
