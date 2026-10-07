# Q04 — `Math.log`'un JVM intrinsic'i ile bit eşitliği

**Amaç:** Minecraft'ın Gauss kaynağı (`MarsagliaPolarGaussian`) `Math.log` kullanır; HotSpot x86_64'te
bu, fdlibm (`StrictMath.log`) değil, JVM'in kendi `dlog` stub'ıdır. Ölçüm (2026-10-08, OpenJDK
25.0.4.1): rastgele 2M girdide %7'si fdlibm'den, örneklenen farklılardan 12/2000'i Rust `f64::ln`'den
bir ULP farklı. `crates/voxel-core/src/random.rs` `strict_log` kullanıyor; bu yüzden
`movement/particle-engine` satır 6 (paket parçacığının Gauss yayılımı) bir ULP sapıyor ve kapı
kırmızı.
**Bağımlılık:** — . **Boy:** M. **Engel:** stub'ın tabloları ve polinomu OpenJDK'dan (GPLv2) gelir;
proje MIT/Apache. Kopyalama/port lisans kararı kullanıcıdadır. Alternatif: algoritmayı yayımlanmış
Intel LIBM belgelerinden bağımsız yeniden kurmak ve tabloyu oracle'dan (JVM'den) üretilmiş veri
olarak almak (AGENTS "formülü yeniden üretemiyorsan tabloyu veri olarak al").

## Oku
- `crates/voxel-core/src/math/log.rs`, `crates/voxel-core/src/random.rs` (`next_gaussian`).
- Java: `world/level/levelgen/MarsagliaPolarGaussian.java`.

## Yapılacaklar
1. Lisans kararı. 2. `jvm_log` (tablo + polinom) ve `next_gaussian`'ın ona geçmesi. 3. JVM'den
milyonlarca girdiyle karşılaştırma (`math` ailesi).

## Kabul
`python3 tools/voxel.py check` → `CHECK OK` (particle-engine dahil).

## Devir notu
