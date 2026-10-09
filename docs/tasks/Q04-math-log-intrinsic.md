# Q04 — `Math.*` intrinsic'leri ve oracle JVM'i

**Amaç:** HotSpot x86_64'te `Math.sin/cos/log/pow` JVM'in kendi stub'larıdır; fdlibm'den ve diğer
platformlardan son bitte ayrılır (ölçüm 2026-10-09, OpenJDK 25.0.4.1: rastgele girdilerde %6'sı).
Parçacık benzetimleri bunu konumlara taşıdığı için `movement/particle-*` kapıyı kırıyordu.
**Karar (kullanıcı):** stub'ı taklit etmek yerine oracle JVM'i `-XX:+UnlockDiagnosticVMOptions
-XX:-UseLibmIntrinsic` ile çalıştırılır (`tools/behavior/run.py` `JVM_FLAGS`, kimliğe girer); böylece
`Math.*` = `StrictMath.*` ve her makinede aynı bit olur. Rust tarafı fdlibm portlarını kullanır
(`photon-core/src/math/{log,fdlibm}.rs`; `StrictMath` ile 600 bin girdide bit eşit).
**Bağımlılık:** — . **Boy:** M.

## Devir notu
Yapıldı: oracle bayrağı; `fdlibm_sin/cos/pow`; `leaves.rs` ve `level_events{,_more}.rs` onlara geçti. Kapı: tüm aileler yeşil.
Sapma: gerçek bir x86-64 JVM'in intrinsic'i son bitte farklı olabilir; oracle bilerek fdlibm'e sabitlendi. `sin/cos` 2^19·π/2 üstü argümanda platform kütüphanesine düşer (hiçbir benzetim ulaşmaz).
Açık: yeni `Math.*` kullanan Rust kodu `fdlibm_*` çağırmalı; platform `sin/cos/powf` kullanan eski yerler (travel, boat, elytra, display, dragon...) henüz kendi oracle'larında sınanmadı.
