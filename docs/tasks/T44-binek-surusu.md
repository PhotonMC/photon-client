# T44 — Binek sürüşü: `PlayerInput`, `PlayerCommand`, `MoveVehicle`, `PaddleBoat`

**Amaç:** Oyuncunun araca/hayvana binmesi ve sürmesi: `Interact` ile binme (T26), `SetPassengers` (var), sürüş girdisi (`PlayerInput`/`PlayerCommand`), tekne kürekleri (`PaddleBoat`), araç konumu (`MoveVehicle`), at kontrolü (zıplama gücü), domuz/deve/mantar.
**Bağımlılık:** T38, T31. **Boy:** L. **Türler:** tekne, minecart, at ve eşekgiller, domuz (eyerli), deve, strider.

## Oku
- Java: `client/player/LocalPlayer.java`.
- Rust: `crates/voxel-world/src/entity.rs` (tekne/yolcu dalları, `set_boat_input`), `crates/voxel-world/src/local_player.rs`, `crates/voxel-net/src/connection/play.rs` (`keys`/tik gönderimi), `docs/client-world-entity-boat-control-conformance.md` (tekne denetim kaydı).

## Yapılacaklar
1. Tekne/yolcu dalları üstünde (`docs/client-world-entity-boat-control-conformance.md`): araç kontrolü `LocalPlayer.rideTick` dalı.
2. Paketler: `ServerboundPlayerInput`, `ServerboundPlayerCommand`, `ServerboundPaddleBoat`, `ServerboundMoveVehicle` ve karşılıkları.
3. Yolcunun kamera/poz kuralları (`getPassengerRidingPosition`, T21'de var).

## Oracle
`MovementOracle.java` `local_player_ticks` kalıbı: gerçek `LocalPlayer` bir araca bindirilir, girdi tikleri koşturulur ve gönderilen paketler bayt bayt karşılaştırılır (araç konumu/`PlayerInput` değişimi).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
At eğitimi/iksir etkileri, ses.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Çok paketli; tekne (T38) ve at (T31) çizilmeden anlamlı sınanamaz. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Bölündü: tekne sürüşü [T44a](T44a-tekne-surusu.md) (bitti), öteki araçlar [T44b](T44b-binek-hayvanlar-minecart.md).
