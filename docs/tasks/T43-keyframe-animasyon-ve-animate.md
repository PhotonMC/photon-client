# T43 — `KeyframeAnimation` oynatıcısı, `Animate` paketi ve `EntityEvent` çizim durumları

**Amaç:** Keyframe tabanlı model animasyonlarını (`KeyframeAnimation`, `AnimationState`, `AnimationDefinition`) ve onları sürecek durumları ekle: `Animate` paketi (kol sallama/uyanma/kritik vuruş) ve `EntityEvent` ile açılan istemci durumları (yeme, silkeleme, tepki).
**Bağımlılık:** T24. **Boy:** L. **Türler:** kullananlar: warden, kurbağa, aksolotl, deve, sniffer, armadillo, copper golem, creaking, breeze, allay (T35/T37/T31 bunu bekler).

## Oku
- Java: `client/animation/KeyframeAnimation.java`, `client/animation/AnimationDefinition.java`, `client/model/AnimationUtils.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (model bölümlerinin pozu), `crates/voxel-net/src/connection/play.rs` (paket işleme; `PLAY_CLIENTBOUND_*` kimlikleri `protocol.rs`'te), `crates/voxel-world/src/living.rs` (istemci tik durumları), `crates/voxel-data/src/entity_model.rs`.

## Yapılacaklar
1. Animasyon tanımları tablo olarak oracle'dan (`AnimationDefinition` statik alanları → üretilmiş Rust tablosu, her parity koşusunda karşılaştırılır).
2. `KeyframeAnimation.apply` ve `AnimationState.updateTime/start/stop` oracle'ı gerçek sınıflarla.
3. `Animate` (uzak oyuncu kol sallaması `swingTime`) ve `EntityEvent` durum eşlemesi; hangi olay hangi animasyonu başlatır (`handleEntityEvent` kolları) bir tablo.

## Oracle
`EntityOracle.java`: (1) `AnimationDefinition` statik alanları tabloya (`tools/generate/`), (2) `KeyframeAnimation.apply` gerçek model bölümü üzerinde zaman satırları, (3) `AnimationState` başlat/durdur/zaman satırı; `living_entity_ticks` kalıbıyla `EntityEvent`/`Animate` paketleri gerçek `ClientPacketListener` işleyicilerine.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Tüm mob animasyon tanımları (ilgili aile görevlerinde kullanılır).

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Altyapıdır: T31 (deve), T35, T37 bunun üstüne yazılır. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı (bölündü): `Animate` paketi (kimlik 2; eylem 0/3 sallama), `LivingEntity.swing`/`updateSwingTime`/`getAttackAnim` (`living::SwingState`; yalnız oyuncu ve `Monster` ilerletir, `monster` türün tablosunda), `EntityEvent` 10 (koyun yemesi), `entity_snapshot.attack_time/attack_arm_left` uzak oyuncu ve insansı mob çizimine bağlandı; sabit süre 6. Kayıt: `docs/client-animate-swing-parity-conformance.md`.
- Oracle: `movement/living-entity-animations` 14 satır (gerçek `ClientPacketListener` + gerçek tik; tavuğun gerçek `aiStep`'i dahil) ve `entity-pick-table`'a `monster`.
- Sapma: sallama süresi eşya/etki okumaz (mızrak, acele), eylem 2/4/5 yok.
- Açık: `KeyframeAnimation` + `AnimationState` oynatıcısı (T43b).
