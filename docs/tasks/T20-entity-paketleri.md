# T20 — Entity paketleri canlı oturumda

**Amaç:** Entity paketleri bugün yalnız probe'larda (`voxel-net/src/connection/probe.rs`) dünya durumuna uygulanıyor; canlı `play` oturumu (`connection/play.rs`) hiçbirini işlemiyor. Aynı ayrıştırıcı ve `ClientWorld::apply_entity*` çağrılarını canlı oturuma bağla.
**Bağımlılık:** yok. **Boy:** L.

## Oku
- Rust: `crates/voxel-net/src/connection/protocol.rs` (ayrıştırıcılar zaten var: `parse_add_entity_packet` ~2320, `parse_set_entity_motion_packet`, `parse_set_passengers_packet`, `parse_entity_metadata_packet`, `parse_entity_position_sync_packet`, `parse_entity_teleport_packet`, `parse_move_entity_*`; paket kimlikleri ~50–90 satırlarında: Add 1, PositionSync 35, MovePos 53/54/56, RotateHead 83, Motion 101, Passengers 107, SetEntityData 99, Teleport 125, BundleDelimiter 0), `crates/voxel-net/src/connection/probe.rs` (`apply_entity` ~2117, `apply_entity_motion` ~2173, `apply_entity_rotation` ~2198, `remove_entities` ~2217, teleport ~2407, move ~2468 — **uygulama çağrı kalıbı**), `crates/voxel-net/src/connection/play.rs` (`PlaySession`, paket `match`, `drain()` ~410–430; mevcut örnek kol: `PLAY_CLIENTBOUND_CHUNK_PACKET_ID` ~57), `crates/voxel-world/src/lib.rs` (`apply_entity`, `remove_entities`, `entities` ~85).
- Java: `.data/minecraft/source/net/minecraft/network/protocol/game/{ClientboundAddEntityPacket,ClientboundBundlePacket,ClientboundRemoveEntitiesPacket,ClientboundEntityEventPacket,ClientboundAnimatePacket,ClientboundTakeItemEntityPacket,ClientboundSetEquipmentPacket,ClientboundSetEntityLinkPacket}.java`; `ClientPacketListener.handleAddEntity` (`grep -n "handleAddEntity\|handleBundlePacket\|handleRemoveEntities"`).

## Yapılacaklar
1. `play.rs` paket `match`'ine entity kollarını ekle: ayrıştır → `self.trace.world.apply_*` (probe'daki çağrı kalıbının aynısı; kopya yok: ortak kısmı `connection/entities.rs`'e al ve hem probe hem play oradan çağırsın — probe'daki 400+ satır tekrarı bu görevle azalır).
2. `Bundle`: `BundleDelimiter` arası paketler tek tik içinde uygulanır (Java `handleBundlePacket` — yalnız sıra garantisi; ayrıştırma sırayla yeterliyse delimiter'ı yok say ve belgele).
3. `RemoveEntities` (kimlik çöz: `protocol.rs`'de yoksa ekle; Java `ClientboundRemoveEntitiesPacket`), `EntityEvent`, `Animate` yalnız ayrıştırılır (durum modeli T22+); bilinmeyen/ayrıştırılmayan entity paketi bağlantıyı kesmez.
4. `PlaySession`'a `entity_count()`/`entity(id)` sorgusu (test ve canlı doğrulama için).

## Oracle
Yeni ailesiz: probe'lar ve `entity` ailesi zaten `ClientWorld::apply_entity*`'ı doğruluyor. Canlı kanıt: `server-conformance`.

## Kabul
`tools/voxel.py check` → `CHECK OK` (parity sayısı sabit); birim test: sahte paket baytlarından `PlaySession` entity sayısı. Canlı (`shot`, `--rcon "summon pig ~ ~ ~"` ardından bekle): log satırında `entities=1` (log biçimini bu görevde ekle).

## Kapsam dışı
Çizim, tick/interpolasyon canlı döngüsü (T21), oyuncu bilgisi eşlemesi.

## Belgeler
`docs/remaining-work.md` M4 madde 1 ve §1 paket kapsam tablosu; `docs/live-play-session.md` kısa not.

## Devir notu
- Yapıldı: `connection/entities.rs` (add, motion, rotate_head, remove, passengers, metadata, position_sync, teleport, step); probe aynı yardımcıları çağırır (yinelenen 120 satır gitti), `play.rs` yeni kollar (ayrıştırılamayan entity paketi yok sayılır; `SetEntityData` da artık gevşek), `PlaySession::entity_count`, oturum sonu `entities=` satırı, birim test (sahte paketlerden sayı, ekleme/adım/bozuk paket/silme).
- Canlı: yeni oyuncu adıyla (`--user Fresh7`) `entities=126`. Kayıtlı `Miner` oyuncusuyla sunucu hiç yığın göndermiyor (kayıtlı oyuncu verisi; istemci hatası değil, T14'ten önceki derleme de aynı); yeni ad kullan.
- Sapma: `Bundle` ayrı ele alınmıyor (sıra korunuyor); `Animate`, `SetEquipment`, `SetEntityLink`, `TakeItemEntity` işlenmiyor.
- Açık: entity tick/interpolasyon (T21).
