# P08/P11 — Vanilla 26.2 ilk oyuncu konumu doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative ve peaceful ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example connection_probe -- 127.0.0.1:25566 RustPosition1
cargo run --offline --example connection_probe -- 127.0.0.1:25566 RustPosition2
```

İki bağımsız Rust koşusu aynı ilk konum düzeltmesi ve loaded bildirimini tamamladı:

| Gözlem | RustPosition1 | RustPosition2 |
| --- | ---: | ---: |
| login finished | `true` | `true` |
| configuration finished | `true` | `true` |
| play login | `true` | `true` |
| player position observations | `[{teleport_id:1, relative_flags:0}]` | `[{teleport_id:1, relative_flags:0}]` |
| player loaded sent | `true` | `true` |
| chunk packet | 22 | 21 |
| decoded chunk | 22 | 21 |
| chunk batch finished | 5 | 5 |
| received packet | 418 | 367 |

Probe çıktısında iki oturumda da packet ID `72` ile gelen teleport ID `1` ve boş relative flag görüldü. Rust, Java kaynak akışındaki sırayla teleport kabulü ve `move_player_pos_rot` cevabını yazdı; ardından `player_loaded` packet'ini gönderdi. Vanilla server logu iki kullanıcı için `logged in`, `joined the game` ve normal probe kapanışında `left the game` kayıtlarını verdi; protocol error veya timeout oluşmadı.

## Açık kapsam

Bu sonuç ilk server correction packet'inin decode edilip cevaplanmasını ve gerçek vanilla sunucusunun bağlantıyı kabul ettiğini doğrular. İlk konumun Java client entity state'ine uygulanması, input/tick hareketi, movement packet eşikleri, collision, teleport prediction, block action, inventory, kamera ve renderer henüz açık kalır. `player_loaded` bildiriminin probe zamanlaması level renderer hazır olma davranışının yerine geçmez.

Sözleşme ayrıntıları [player position contract belgesindedir](player-position-contract.md). Chunk/heightmap/light wire kanıtı [ayrı P08 kaydındadır](chunk-packet-parity.md). Bu test resmi vanilla sunucusuyla yapıldı; Pumpkin koşusu bu baseline'a karıştırılmadı.
