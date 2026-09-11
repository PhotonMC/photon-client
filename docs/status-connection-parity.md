# P06.a — Vanilla 26.2 status socket doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** resmi vanilla 26.2 dedicated server, offline yerel koşu
**Sonuç:** Rust status probe iki ardışık koşuda status response ve ping echo akışını tamamladı.

## Çalıştırma

```text
cargo run --offline --example status_probe -- 127.0.0.1:25566
```

Komut iki bağımsız TCP oturumunda çalıştırıldı. İki sonuç da aynıydı:

```text
status_connection=ok
status_response_length=115
status_response={"description":"Vanilla 26.2 parity test","players":{"max":20,"online":0},"version":{"name":"26.2","protocol":776}}
pong_time=72623859790382856
received_packet_count=2
```

`72623859790382856`, `0x0102030405060708` değerinin signed 64-bit gösterimidir. Status response içindeki sürüm ve protocol alanları sabit hedefle eşleşti; server ping yanıtından sonra bağlantıyı kapattı.

## Açık kapsam

Bu sonuç status intention, status request/response framing, UTF-8 response payload'ı, ping/pong echo'su ve status oturumunun kapanışını kapsar. `ServerStatus` JSON şemasının Rust'a ayrıştırılması, favicon, server list ekranı, login/play bağlantısı ve online authentication açık kalır.

Bu kayıt vanilla baseline içindir; Pumpkin koşusu [ayrı oyun oturumu kaydında](game-connection-parity.md) tutulur. Orijinal Java dosyaları ve ham server logları yerel referans/geçici dizinlerde kaldı.
