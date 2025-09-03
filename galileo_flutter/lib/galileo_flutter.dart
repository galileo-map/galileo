library;

import 'dart:ffi' as ffi;

export 'package:galileo_flutter/src/galileo_map_widget.dart' show GalileoMapWidget;


import 'src/rust/api/simple.dart' as simple;
import 'src/rust/frb_generated.dart' as rlib_gen;

export 'package:galileo_flutter/src/rust/api/dart_types.dart' show MapViewport, MapSize, LayerConfig;

Future<void> initGalileo() async {
  await rlib_gen.RustLib.init();
  simple.galileoFlutterInit(
        ffiPtr: ffi.NativeApi.initializeApiDLData.address,
      );
      
}
