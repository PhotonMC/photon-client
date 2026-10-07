# T44c4 — Minecart yolcusu ve binme/inme

**Amaç:** minecart yolcusu (`AbstractMinecart`, koltuk ve sunucunun yürüttüğü hareket) ve binme/inme akışı (`startRiding`, `getDismountLocationForPassenger`, `mount.onboard`, `Shift` ile inme).
**Bağımlılık:** T44c3. **Boy:** M.

## Oku
- Java: `world/entity/vehicle/minecart/AbstractMinecart.java`, `Entity.java` (`startRiding`, `removeVehicle`, `getDismountLocationForPassenger`).
- Rust: `voxel-world/src/{vehicle_ride,mount_ride,local_player}.rs`.

## Oracle
`movement/ride-camel` örneğiyle `ride` satırları; `kind` olarak minecart.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/records/client-mount-ride-parity-conformance.md`.

## Devir notu
Minecart: `ride-minecart` 6 satır gerçek Minecart/MinecartChest ile bit bit eşit (duran araç; kayıt conformance belgesinde).
Binme/inme: `mount.onboard` eylem çubuğu iletisi, araç yolcusuyla silinince `Teleport`ın oyuncuya uygulanması
(`removedPlayerVehicleId`) ve yolcuya `PlayerPosition` uygulanmaması — `play.rs` birim testi, oracle yok.
Oracle'da `enabledFeatures` aracı yaratmadan önce kurulmalı (minecart kurucusu okur).
Sapma: inme konumu sunucuda; dolaylı yolcu zinciri ve `mount.onboard` ekran görüntüsü yalnız canlıda (domuz) görüldü.
Açık: minecart'ın kendi hareketi T38c.
