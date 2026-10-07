# P01.c — BlockPos breadth-first traversal sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos grafiğini Java'nın bounded breadth-first traversal sırasıyla yürütür.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:483-522` ve `736-740` incelendi. Başlangıç düğümü depth 0 ile FIFO kuyruğa alınır; her çekişte `asLong()` packed anahtarı visited kümesinde bir kez işlenir. Node processor sonucu `SKIP` ise düğüm sayılmaz ve komşu provider çağrılmaz; `STOP` traversal'ı count artırmadan bitirir. `ACCEPT` count'u artırır, `maxCount` kontrolünü geçerse hemen döner ve `depth < maxDepth` ise komşuları depth+1 ile kuyruğa ekler.

## Davranış sözleşmesi

- Kuyruk FIFO'dur; komşular provider'ın verdiği sırayla eklenir.
- Visited anahtarı BlockPos koordinatlarının packed long biçimidir; packed alanda çakışan koordinatlar aynı düğüm sayılır.
- Processor her yeni packed anahtar için provider'dan önce çağrılır.
- `SKIP` count'a ve provider'a etkisizdir; `STOP` count'a ve provider'a etkisiz biçimde traversal'ı keser.
- `ACCEPT` sonrası count bir artırılır; `count >= maxCount` kontrolü komşu provider'ından önce yapılır.
- Yalnızca `depth < maxDepth` olan kabul edilmiş düğümler komşu ekler.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_breadth_first_traversal"`, `start`, `max_depth`, `max_count` ve position/status/neighbors kayıtlarını taşır. Sonuç `count`, processor'ın gördüğü `processed` sırası ve provider çağrılarının `provider_calls` sırasını içerir.

Java oracle ile Rust adapter aynı sentetik graph kayıtlarını tüketir. Rust karşılığı `BlockPos::breadth_first_traversal` ve `TraversalNodeStatus` yordamlarıdır.

## Kapsam dışı

Gerçek dünya komşuluk üretimi, chunk/entity graph'ı, MutableBlockPos API'si ve packed codec yolları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_breadth_first_traversal` işlemi, Java'nın FIFO kuyruk ve packed visited traversal'ını sentetik grafiklerle karşılaştırır.

Java math capture kimliği `3727446d35576418209105f5c151ee1e3c8b99eb95e0ef13e2b16e640c2195c1` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt processor/provider sırasını, FIFO komşu ekleme düzenini, SKIP/STOP dallarını, depth/count sınırlarını ve packed long visited anahtarını kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `48c013300094ef4882ade1cee5d8ac9569b681c9ca3f31ddcc1fa275e3a1e44d`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
