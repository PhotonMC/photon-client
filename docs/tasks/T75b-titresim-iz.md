# T75b — Titreşim ve iz parçacıkları

**Amaç:** `VibrationSignalParticle` (`VibrationParticleOption`: hedef konum kaynağı — blok ya da
entity — ve varış süresi; yönelen dörtgen) ve `TrailParticle` (`TrailParticleOption`: hedef, renk,
süre). İkisi de ağdan konum kaynağı/hedef okur; `ParticleData`'ya eklenir.
**Bağımlılık:** T75. **Boy:** M.

## Oku
Java: `client/particle/{VibrationSignalParticle,TrailParticle}.java`,
`core/particles/{VibrationParticleOption,TrailParticleOption}.java`,
`world/level/gameevent/PositionSource*.java`. Rust: `crates/photon-world/src/particles/attract.rs`.

## Oracle
`particle-ticks` satırları (gerçek sağlayıcılar, seçenekler satırdan).

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-particle-attract-parity-conformance.md`.

## Devir notu

`PositionSource` (blok/entity) ve `ParticleData::{Vibration,Trail}` `photon-world`'de; tikleri
`crates/photon-world/src/particles/vibration.rs`'de (`attract.rs` ayrı bırakıldı — bu ikisi bir
yere değil bir hedefe gider, kendi tick'leri var). Ağ çözümü `photon-net` (`PositionSource`'un kayıt
kimliği: `BLOCK`=0, `ENTITY`=1, Java kayıt sırası). Oracle: `MovementParticleOracle`'ın `registered`
listesine ve `optionsOf`'a eklendi (gerçek sağlayıcı yolu); 4 yeni `particle-ticks` satırı.

Kanıt: `python3 tools/photon.py parity --family movement` yeşil (yalnız bilinen Q04 farkı).
`photon-net`'in "bilinmeyen tür" testi örneğini `vibration`'dan (artık tanınıyor) `shriek`'e
taşıdım.

Sapma/açık (kayıtta ayrıntılı): titreşimin entity hedefli durumu oracle'da yok (fikstür
gerektirirdi); titreşimin döndürülmüş çift dörtgen çizimi bağlanmadı (dünya tarafı kanıtlı, render
genel kamera-dönük dörtgenle çiziyor).
