# T43b — `KeyframeAnimation` oynatıcısı ve `AnimationState`

**Amaç:** Keyframe tabanlı model animasyonlarını (`KeyframeAnimation.apply`, `AnimationDefinition`, `AnimationState.start/stop/updateTime`) çalıştır; ilk tüketici (T31 deve, T35 aksolotl/kurbağa, T37 warden…) bununla yazılır.
**Bağımlılık:** T43. **Boy:** M–L. İlk tüketici görevinin içinde yapılması önerilir (tüketicisiz altyapı yazma, YAGNI).

## Oku
- Java: `client/animation/KeyframeAnimation.java`, `client/animation/AnimationDefinition.java`, `client/animation/definitions/` (kullanılacak aileninki), `world/entity/AnimationState.java`, `client/model/AnimationUtils.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (parçaların pozu, `child_mut`), `crates/voxel-world/src/animal.rs` (durum kalıbı), `crates/voxel/src/mobs.rs`.

## Yapılacaklar
1. `AnimationDefinition` tablosu (kullanılan aile için) oracle'dan, her koşuda karşılaştırılır; `KeyframeAnimation.apply` Rust'ta (doğrusal ve Catmull-Rom).
2. `AnimationState` (süre, başlat/durdur, hız) ve onu süren `EntityEvent` kolları.

## Oracle
`EntityOracle.java`: gerçek model parçası üzerinde `KeyframeAnimation.apply` zaman satırları; gerçek `AnimationState` başlat/durdur satırları; durum geçişleri `movement/living-entity-animations` kalıbıyla paketten.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Tüm mob animasyon tanımları (aile görevlerinde).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
