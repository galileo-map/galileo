library;

import 'dart:ffi' as ffi;

export 'package:galileo_flutter/src/galileo_map_widget.dart' show GalileoMapWidget;

import 'src/rust/api/api.dart' as rlib;
import 'src/rust/frb_generated.dart' as rlib_gen;

export 'package:galileo_flutter/src/rust/api/dart_types.dart'
    show MapViewport, MapSize, LayerConfig, MapInitConfig;

Future<void> initGalileo() async {
  await rlib_gen.RustLib.init();
  rlib.galileoFlutterInit(ffiPtr: ffi.NativeApi.initializeApiDLData.address);
}
