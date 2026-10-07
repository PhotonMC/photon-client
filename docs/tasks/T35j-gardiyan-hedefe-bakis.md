# T35j — Gardiyanın hedefe bakışı ve karadaki çırpınışı

**Amaç:** `Guardian.aiStep` kalan dalları: hedef varken `LookControl.setLookAt(hedef, 90, 90)` + `tick()` (baş ve eğim hedefe döner), `setYRot(yHeadRot)`, kabarcık parçacıklarının aynı rastgele kaynaktan çekilişleri (hareketli ve suda: iki tur üç `nextDouble`; hedef varken: mesafe döngüsü), karada `onGround` dalı (rastgele itme ve dönüş, üç `nextFloat`).
**Bağımlılık:** T35i. **Boy:** M.

## Oku
- Java: `world/entity/monster/Guardian.java` (`aiStep`), `world/entity/ai/control/LookControl.java`, `Mob.java` (`aiStep`, `baseTick`), `LivingEntity.aiStep` (yerel dönüş ve konum enterpolasyonu).
- Rust: `crates/voxel-world/src/guardian.rs` (başlıkta eksik çekilişler), `crates/voxel-world/src/entity.rs` (`tick_guardian`, `tick_living_pose`, `tick_head_rotation`).

## Oracle
Gerçek `Guardian` tiki için `living-entity-animations`; hedefin oracle sahnesinde gerçek bir varlık olması gerekir (şimdiki `living()` satırı tek varlık kurar; ikinci varlık için sahne yardımcısı genişletilmeli).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacık çizimi, saldırı davranışı.

## Devir notu
- Yapıldı: `living::look_at_tick` (`LookControl`), `GuardianState::tick` hedefe bakış, kabarcık çekilişleri, karada çırpınış, ortam sesi zarı; hedef dünyadan çözülür (`guardian_target_of`). Kayıt: `docs/records/client-guardian-look-parity-conformance.md`.
- Bulgu: javac `180F / (float)Math.PI`'yi float katlıyor; dekompile kaynak yanıltıcı, `javap -c` ile okundu. Ortam sesi zarı (`baseTick`) gardiyanın kaynağını gizlice kaydırıyordu; ancak rastgele durum karşılaştırılınca göründü.
- Sapma: yerel oyuncu hedefi yalnız birim testle; hasar, ölüm, adım sesi çekilişleri ve hasarda ortam süresi sıfırlanması yok (ses görevleri).
