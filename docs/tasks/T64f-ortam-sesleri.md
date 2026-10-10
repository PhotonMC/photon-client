# T64f — Ortam sesleri

**Amaç:** [T64d](T64d-ses-tetikleri-ortam-muzik.md)'nin kalanı: mağara ve su altı döngüleri (`UnderwaterAmbientSoundHandler`, `BiomeAmbientSoundsHandler`: `minecraft:audio/ambient_sounds` özniteliği), olay sesleri, eşya kullanım sesleri, binekte düşüş sesleri, ekran müziği (`Screen.getBackgroundMusic`) ve ejderha müziği.
**Bağımlılık:** T64e. **Boy:** L.

## Oku
Java: `client/resources/sounds/{Underwater,BiomeAmbient}*Handler.java`, `client/Minecraft.java` (`getSituationalMusic`). Rust: `crates/photon/src/music.rs`, `crates/photon-world/src/local_player/sounds.rs`.

## Kabul
`CHECK OK`.

## Devir notu
(boş)
