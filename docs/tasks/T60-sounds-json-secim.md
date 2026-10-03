# T60 — `sounds.json` çözümü ve ağırlıklı ses seçimi

**Amaç:** `sounds.json` (asset indeksindeki `minecraft/sounds.json`) çözümü, `WeighedSoundEvents` ve `Sound` ağırlıklı seçim (rastgele sayı akışı dahil), `replace`/`stream`/`preload`/`attenuation_distance`/`volume`/`pitch` alanları.
**Bağımlılık:** T24 ile bağımsız (bağımlılık yok). Ses yığını kararı: [`audio-backend-decision.md`](../audio-backend-decision.md) (çıkış `cpal`, çözücü `lewton`).

## Oku
- Java: `client/sounds/{SoundManager,WeighedSoundEvents,Weighted}.java`, `client/resources/sounds/{Sound,SoundEventRegistration,SoundInstance,AbstractSoundInstance}.java`, `sounds/{SoundEvent,SoundSource}.java`.
- Rust: `crates/voxel-game/` (kayıt/dil kalıbı), `crates/voxel/src/assets.rs` (asset indeksi okuma), `crates/voxel-core/src/random*` (P01.e rastgele yolları).

## Yapılacaklar
1. Çözümleyici ve seçim (`SoundManager.getSoundEvent`, `WeighedSoundEvents.getSound`): ağırlık toplamı, `Sound.getWeight`, iç içe olay başvurusu.
2. Ses olayı kayıt defteri adı → olay haritası (yerel `SoundEvents` adları sunucudan kimlikle de gelir, bkz. T61).

## Oracle
Gerçek `SoundManager` ayrıştırması headless (`Bootstrap` + sahte kaynak yöneticisi) ve aynı tohumla seçim dizileri; `EntityOracle` değil yeni `SoundOracle.java` ve `sound` ailesi.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Ses çıkışı ve çözme (T62/T63), sunucu paketleri (T61).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
