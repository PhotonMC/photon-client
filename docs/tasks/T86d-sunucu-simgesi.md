# T86d — Sunucu simgesi (favicon)

**Amaç:** Sunucu listesinde her sunucunun kendi 64×64 simgesi (`FaviconTexture`): durum yanıtındaki
favicon (`ServerData.validateIcon`: PNG, en çok 1024×1024) satırda çizilir ve `servers.dat`'a base64
olarak yazılır (`onPersistentDataChange` → `ServerList.save`); kayıtlı simge ping beklenirken de
çizilir. Bugün her satır `unknown_server` simgesini çiziyor (T86b).
**Bağımlılık:** T86b. **Boy:** M.

## Oku
- Java: `client/gui/screens/FaviconTexture.java`, `ServerSelectionList.OnlineServerEntry`
  (`uploadServerIcon`, `extractIcon`), `client/multiplayer/ServerData.java` (`validateIcon`).
- Rust: `crates/photon-render/src/gui/screen/servers.rs`, `crates/photon-render/src/gui/atlas.rs`
  (dinamik doku yok; GUI çizimine ayrı doku yuvası gerekir), `crates/photon-game/src/servers.rs`.

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
- Yapıldı: base64 kodlayıcı ve SHA-1 (`photon-core`), `ServerData` simge işlemleri ve `validateIcon` (`photon-game/servers.rs`), atlasta 32 simge yuvası (`GuiAtlas::set_favicon`, `gui/favicon.rs`), Vulkan'da atlasın yeniden gönderilmesi (`replace_gui_texture`), menüde ping → kaydet → çiz (`menu/icons.rs`). G: `gui/server-list-icons` (10 satır) yeşil; sahte durum sunucusuyla canlıda simge görüldü ve `servers.dat`'a yazıldı.
- Sapma: simge sayısı 32 ile sınırlı (Java sınırsız); fazlası bilinmeyen simgeyle çizilir. Atlas yeni simgede baştan gönderilir (`device_wait_idle`), nadir olay.
- Oracle: Java'da `DynamicTexture` GPU ister; yalnız 64×64 başlıklı PNG için GPU'suz doku takılır, geri kalan `FaviconTexture.upload` gerçek.
- Açık: LAN girdileri, dünya (`forWorld`) simgeleri; kapıdaki `movement/particle-engine` farkı Q04'ten, bu görevden değil.
