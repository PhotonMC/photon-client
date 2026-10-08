# T95a — Görüş mesafesi seçeneği

**Amaç:** Video ayarlarındaki "Render Distance" kaydırıcısını oyuna bağlamak: `options.txt`
`renderDistance` (2–32, varsayılan 12), sunucuya giden `ClientInformation` (bağlanırken ve seçenekler
kaydedilince `broadcastOptions`, yalnız değiştiyse), çizim sınırı `getEffectiveRenderDistance`
(seçenek, sunucununkinden fazla değil; `SetChunkCacheRadius` sunucunun değerini günceller).
**Bağımlılık:** T07. **Boy:** S.

## Oku
- Java: `client/Options.java` (`renderDistance`, `buildPlayerInformation`, `broadcastOptions`,
  `getEffectiveRenderDistance`), `ClientPacketListener.broadcastClientInformation`,
  `handleSetChunkCacheRadius`.
- Rust: `crates/photon-render/src/gui/options/video.rs`, `crates/photon-net/src/connection/session.rs`,
  `crates/photon/src/play/draw.rs`.

## Kabul
`CHECK OK`; aynı sunucuda 6 ve 16 chunk ile gelen chunk sayısı farklı.

## Devir notu
Yapılan: `Options.render_distance` (`renderDistance`), video kaydırıcısı seçenekten okuyup yazar;
`ClientInformation` seçeneklerden (`controls::client_information`) kurulur, bağlanırken ve her
kayıtta gönderilir (`PlaySession::send_client_information`, play id 14, değişmediyse gönderilmez);
çizim mesafesi seçenek ile sunucu değerinin küçüğü; `SetChunkCacheRadius` sunucu değerini günceller.
Canlı ölçüm: aynı sunucuda 6 chunk ile 213, 16 ile 1057 chunk geldi. Oyun sonu satırı `chunks=` yazar.
Sapma: dil, sohbet görünürlüğü dışındaki bilgi alanları (skin parçaları, ana el, filtre, parçacık)
hâlâ varsayılan; sunucu değeri değişince bölümler hemen yeniden sıralanmıyor.
