# T36c — Papağan

**Amaç:** `ParrotModel` (konumlar: ayakta, uçan, oturan, parti, omuzda; kanat çırpma durumu) ve çeşit dokuları. Breeze T36e'ye, mutlu ghast T36f'ye ayrıldı.
**Bağımlılık:** T36. **Boy:** M.

## Oku
- Java: `client/model/animal/parrot/ParrotModel.java`, `client/renderer/entity/ParrotRenderer.java`, `world/entity/animal/parrot/Parrot.java` (`calculateFlapping`).
- Rust: `crates/voxel/src/monsters.rs`, `crates/voxel-render/src/mob_model.rs`.

## Oracle
`airborne_model` kalıbı; gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, omuz papağanı (oyuncu katmanı), davranışlar; jukebox (parti pozu).

## Devir notu
- Yapıldı: `setup_parrot` (beş poz, `prepare` ve düşen `switch` kolları aynen), `voxel-world/parrot.rs` (kanat çırpışı, çeşit dokuları ve kenetleme), çeşit verisi 20, `monsters.rs` tablosuna papağan (çeşit dokusu, oturan/uçan/ayakta poz). Kayıt: `docs/client-parrot-parity-conformance.md`.
- Bulgu: çeşit verisi 20'dedir (19 değil); `grey` dosyası `gray` çeşidinin dokusu.
- Bölündü: breeze T36e'de; mutlu ghast T36f'de.
- Sapma: parti pozu jukebox olayı bağlanana dek seçilmez; omuz katmanı yok. Canlı gözlem yapılmadı.
