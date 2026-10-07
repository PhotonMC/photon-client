# T64 — Oyun ses tetikleri ve müzik

**Amaç:** Adım, blok kırma/koyma, eşya kullanma, hasar, ortam, yağmur ve kapı/sandık gibi istemci tetiklerini ses olaylarına bağla; `MusicManager` (menü/oyun müziği, bekleme).
**Bağımlılık:** T63; blok ve hareket tetikleri için M1 (bitti), mob sesleri M4. Ses yığını kararı: [`audio-backend-decision.md`](../audio-backend-decision.md) (çıkış `cpal`, çözücü `lewton`).

## Oku
- Java: `client/sounds/MusicManager.java`, `world/level/block/SoundType.java`, `client/player/LocalPlayer.java` (adım sesi), `world/entity/Entity.java` (`playStepSound`, `getSwimSound`), `client/multiplayer/ClientLevel.java` (`playSound`).
- Rust: `crates/voxel-world/src/local_player.rs`, `crates/voxel/src/play.rs`.

## Yapılacaklar
1. `SoundType` tablosu (blok başına ses olayı, ses/perde) oracle'dan üretilir; adım ve kırma tetikleri.
2. Hasar/ölüm, yağmur döngüsü, ortam (mağara, su altı); `MusicManager` seçim ve bekleme.

## Oracle
Gerçek `SoundType` tablosu ve adım/kırma tetikleyicisinin çağrıları; tohumlu müzik seçimi.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Mob ses olayları (aile görevlerinde), altyazılar.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı (T64a): `SoundType` tablosu (oracle'dan), adım/yüzme/vuruş/kırma (`LevelEvent` 2001)/koyma sesleri. Kayıt: `docs/records/client-block-sounds-conformance.md`.
- Kalan → [T64b](T64b-ses-tetikleri-kalan.md): atlama/iniş, hasar, ortam, yağmur, müzik, tetiklerin oracle'ı.
