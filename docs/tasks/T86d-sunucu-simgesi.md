# T86d — Sunucu simgesi (favicon)

**Amaç:** Sunucu listesinde her sunucunun kendi 64×64 simgesi (`FaviconTexture`): durum yanıtındaki
favicon (`ServerData.validateIcon`: PNG, en çok 1024×1024) satırda çizilir ve `servers.dat`'a base64
olarak yazılır (`onPersistentDataChange` → `ServerList.save`); kayıtlı simge ping beklenirken de
çizilir. Bugün her satır `unknown_server` simgesini çiziyor (T86b).
**Bağımlılık:** T86b. **Boy:** M.

## Oku
- Java: `client/gui/screens/FaviconTexture.java`, `ServerSelectionList.OnlineServerEntry`
  (`uploadServerIcon`, `extractIcon`), `client/multiplayer/ServerData.java` (`validateIcon`).
- Rust: `crates/voxel-render/src/gui/screen/servers.rs`, `crates/voxel-render/src/gui/atlas.rs`
  (dinamik doku yok; GUI çizimine ayrı doku yuvası gerekir), `crates/voxel-game/src/servers.rs`.

## Yapılacaklar
1. GUI'ye çalışma anında yüklenen dokular (sunucu başına, 64×64); base64 kodlayıcı.
2. Satır çizimi ve kayıt.

## Oracle
G: `join_multiplayer_screen` satırlarına favicon'lu yanıt (`texture` op adı Java'nın
`servers/<hash>/icon` yolu).

## Kabul
`CHECK OK`; canlı sunucunun simgesi listede.

## Belgeler
`docs/gui-oracle.md` satırı.

## Devir notu
