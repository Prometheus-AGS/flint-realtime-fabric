# Tasks — p18-c009

- [x] Hand-written Dart shim (lib/src/transport.dart or similar) over the sync FFI: connect, subscribe (callback), ack
- [x] Wire the shim through frf_dart.dart so consumers get a working transport API
- [x] Keep the generated frf.dart CRDT surface exported; shim only covers the broken async transport
- [x] Smoke: dart analyze clean (hand-authored), dart pub get resolves, a minimal import test
