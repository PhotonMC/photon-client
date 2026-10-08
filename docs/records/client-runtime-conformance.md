# P06–P08 — Ölçülebilir istemci giriş ucu — [server-conformance] birleşik kayıt

Bu kayıt, `src/main.rs` içindeki ilk çalıştırılabilir istemci yüzeyini ve
vanilla 26.2 oturumundan sonra dünya tick döngüsünün çalıştırılmasını kapsar.
Kanıt sınıfı `server-conformance`'tır: Rust giriş noktası vanilla sunucusuna
bağlanır, gelen chunk'larla `ClientWorld` oluşturur ve bu durumu tick eder.
Java istemcisiyle aynı girdinin state veya frame karşılaştırması yapılmadığı
için bu kayıt `client-parity` değildir.

## Sözleşme

- `photon` binary'si isteğe bağlı olarak `address`, `username`, `ticks`
  ve `timeout_seconds` argümanlarını alır. Varsayılanlar sırasıyla
  `127.0.0.1:25566`, `Photon`, `20` ve `20`'dir.
- Giriş noktası mevcut login/configuration/play bağlantısını kurar ve ilk
  chunk akışından `ClientWorld` durumunu oluşturur.
- Bağlantı döndükten sonra `ticks` kadar `ClientWorld::tick` çağrısı yapılır;
  dünya saatinin başlangıç değeri `0`'dan yürütülen tick sayısına ilerlemesi
  gözlenebilir.
- Render yüzeyi henüz boş olduğu için `rendered_frames=0` yazılır. Pencere,
  input, kamera, entity, ses ve GUI bu kaydın kapsamı değildir.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-12

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Sunucu `/tmp/photon-vanilla-26.2-test` altında `127.0.0.1:25566`, seed `776`,
survival/peaceful, spawn protection `0`, view/simulation distance `5` ve boş
sunucuda 60 saniye pause ayarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --locked --offline --release -- 127.0.0.1:25566 MainRuntimeC26 20 20
```

Çıktı:

```text
client_connection=ok
play_login=true
world_constructed=true
world_chunk_count=25
world_section_count=600
ticks_run=20
world_game_time=20
rendered_frames=0
```

Bu koşu, ana binary'nin gerçek vanilla sunucusuna bağlanabildiğini, chunk
paketlerinin sorgulanabilir dünya kopyasına ulaştığını ve bağlantıdan sonra
tick döngüsünün çalıştığını gösterir. Sunucu oturumunda protocol error
oluşmadı.

## Açık kapsam

Bu kayıt oynanabilir istemciyi veya Java istemcisiyle eşitliği kapatmaz.
Pencere/input olayları, gerçek frame üretimi, ağ paketlerini tick içinde
işleme, oyuncu yerel hareketi, eylem/prediction, entity yaşam döngüsü, ışık
ve block update'leri, kamera, ses, GUI, disconnect/reconnect ve
dimension/respawn hâlâ açık işlerdir.
