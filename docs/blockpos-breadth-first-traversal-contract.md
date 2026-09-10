# P01.c — BlockPos breadth-first traversal sözleşmesi

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
