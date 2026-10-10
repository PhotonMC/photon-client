# T64g — Biyom ve mağara ortam sesleri

**Amaç:** `BiomeAmbientSoundsHandler` (`minecraft:audio/ambient_sounds`: döngü çapraz geçişi 40 tik, ek sesler, mood/mağara sesi), olay sesleri, eşya kullanım sesleri, binekte düşüş sesleri, ekran müziği (`Screen.getBackgroundMusic`) ve ejderha müziği.
**Bağımlılık:** T64f. **Boy:** L (sığmazsa böl).

## Oku
Java: `client/resources/sounds/BiomeAmbientSoundsHandler.java`, `client/Minecraft.java` (`getSituationalMusic`). Rust: `crates/photon/src/music.rs` (öznitelik yükleyici), `crates/photon-world/src/local_player/{underwater,sounds}.rs`, `crates/photon/src/sound.rs` (`handle_underwater` kalıbı).

## Kabul
`CHECK OK`.

## Devir notu
(boş)
