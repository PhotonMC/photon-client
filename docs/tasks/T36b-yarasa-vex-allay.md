# T36b — Yarasa, vex ve allay

**Amaç:** `BatModel` (asılı ve uçan, `BatRenderer`), `VexModel` (kanatlar, yüklenme, ışıklı doku), `AllayModel` (kanatlar, dans) ve ilgili istemci durumları (`Bat.isResting`, `Vex` bayrakları, `Allay` dans verisi). Elde tutulan eşya T36d'ye ayrıldı.
**Bağımlılık:** T36. **Boy:** M.

## Oku
- Java: `client/model/ambient/BatModel.java`, `client/model/animal/allay/AllayModel.java`, `client/model/monster/vex/VexModel.java`, `client/renderer/entity/{Bat,Vex,Allay}Renderer.java` ve durum sınıfları, `world/entity/ambient/Bat.java`, `world/entity/monster/Vex.java`, `world/entity/animal/allay/Allay.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_phantom`, `setup_ghast`: yaşa bağlı kanat kalıbı), `crates/voxel/src/monsters.rs` (`MONSTERS` tablosu, `pose_monster`).

## Oracle
`airborne_model` kalıbı (gerçek model + durum); gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, davranışlar; elde tutulan eşya (T36d).

## Devir notu
- Yapıldı: `setup_bat`, `setup_vex`, `setup_allay`; `voxel-world/{bat,allay}.rs` (yarasanın durumları ve yukarıdaki bloğa yapışması, allay'ın eşya ve dans ilerlemesi); vex yüklenme bayrağı; veriler 16/16/16; `monsters.rs` tablosuna üç tür (vex ve allay'da blok ışığı 15, vex'te yüklenme dokusu). Kayıt: `docs/client-flier-parity-conformance.md`.
- Bulgu: vex ve allay'ın `entityTranslucent` çizimi için yeni boru hattı gerekmedi; entity boru hattı zaten harmanlıyor.
- Bölündü: elde tutulan eşya T36d'de.
- Sapma: yarasanın kutusu izlenmez; doku yolları oracle'sız. Canlı gözlem yapılmadı.
