# T92 — İstatistikler (`AwardStats`, `StatsScreen`)

**Amaç:** İstatistik isteği (`ClientCommand` REQUEST_STATS), `AwardStats` paketi, `StatsCounter`
istemci kopyası ve `StatsScreen` (genel, eşyalar, moblar sekmeleri; sıralama, kaydırma).
**Bağımlılık:** T85. **Boy:** M.

## Oku
- Java: `client/gui/screens/achievement/StatsScreen.java`, `stats/StatsCounter.java`, `stats/StatType.java`,
  `network/protocol/game/ClientboundAwardStatsPacket.java`, `PauseScreen` (Statistics düğmesi).
- Rust: `crates/photon-render/src/gui/screen/pause.rs` (düğme), liste widget'ı.

## Yapılacaklar
1. Paket ve sayaçlar; istek ve "Downloading statistics" ekranı.
2. Üç sekme, sütun sıralama, eşya/mob satırları.
3. G tarifi: `stats_screen`.

## Oracle
G.

## Kabul
`CHECK OK`.

## Kapsam dışı
—

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
