# T35e — Nautilus, zombi nautilus, gardiyan ve elder gardiyan

**Amaç:** `NautilusModel` (+ kabuk/zırh katmanı, zombi çeşidi), `GuardianModel` (diken ve kuyruk salınımı, göz ışın hedefi, ışın çizimi: `GuardianRenderer`), elder gardiyan ölçeği.
**Bağımlılık:** T35b. **Boy:** M–L.

## Oku
- Java: `client/model/animal/nautilus/*.java`, `client/model/monster/guardian/GuardianModel.java`, `client/renderer/entity/{Nautilus,ZombieNautilus,Guardian,ElderGuardian}Renderer.java`, `world/entity/monster/Guardian.java` (`DATA_ID_ATTACK_TARGET`, `clientSideTailAnimation*`, `getAttackAnimationScale`), `world/entity/animal/nautilus/*.java`.

## Oracle
`squid_model` kalıbı; ışın için yakalayıcı toplayıcı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, saldırı davranışı.

## Devir notu
(boş)
