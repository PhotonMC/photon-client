# T64f — Su altı ortam sesleri

**Amaç:** [T64d](T64d-ses-tetikleri-ortam-muzik.md)'nin su altı kısmı: giriş/çıkış sesi, solan döngü, rastlantısal ek sesler (`UnderwaterAmbientSoundHandler`). Mağara/biyom döngüsü [T64g](T64g-biyom-ortam-sesleri.md)'dedir.
**Bağımlılık:** T64e. **Boy:** M.

## Oku
Java: `client/resources/sounds/{Underwater,BiomeAmbient}*Handler.java`, `client/Minecraft.java` (`getSituationalMusic`). Rust: `crates/photon/src/music.rs`, `crates/photon-world/src/local_player/sounds.rs`.

## Kabul
`CHECK OK`.

## Devir notu
Yapıldı: `local_player/underwater.rs` (`update_is_underwater`, ek sesler, `UnderwaterSound`), `SoundSystem::handle_underwater` (döngü solması 40 tik, ek sesler sudan çıkınca durur). Giriş/çıkış sesi `movement` oracle'ında karşılaştırılır (Java adaptörü yalnız bu ikisini ambient süzgecinden geçirir); döngü ve ek sesler şansa bağlı olduğundan yalnız birim testle sınanır. Açık: bkz. T64g.
