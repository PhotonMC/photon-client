# T91 — İlerlemeler (`UpdateAdvancements`, `AdvancementsScreen`)

**Amaç:** İlerleme ağacının istemci kopyası (`ClientAdvancements`: ekleme/silme/ilerleme, sıfırlama),
`AdvancementsScreen` (sekmeler, ağaç düzeni, sürükleme, ipuçları), `SelectAdvancementsTab` iki yönlü,
L tuşu. Bildirim kutusu (toast) T91b'de.
**Bağımlılık:** T85. **Boy:** L.

## Oku
- Java: `client/multiplayer/ClientAdvancements.java`, `advancements/AdvancementTree.java`,
  `advancements/TreeNodePosition.java` (düzen algoritması), `client/gui/screens/advancements/*.java`,
  `network/protocol/game/ClientboundUpdateAdvancementsPacket.java`.
- Rust: `crates/voxel-net/src/connection/play/incoming.rs` (yönlendirme), bileşen/eşya ikonları
  (`crates/voxel-render/src/gui/items.rs`).

## Yapılacaklar
1. Paket çözümü ve ağaç kopyası; `TreeNodePosition` düzeni.
2. Ekran: sekme düğmeleri, ağaç çizimi, sürükleme, ipucu.
3. G tarifi: `advancements_screen`.

## Oracle
G; ağaç düzeni ayrı `math` satırları (gerçek `TreeNodePosition`).

## Kabul
`CHECK OK`; Pumpkin'de L ile ekran açılır.

## Kapsam dışı
Toast (T91b).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
