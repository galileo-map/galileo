library;

import 'dart:ffi' as ffi;
import 'src/rust/api/simple.dart' as simple;
export 'src/rust/api/dart_types.dart';
export 'src/rust/api/galileo_map.dart';
import 'src/rust/frb_generated.dart' as rlib_gen;
export 'src/galileo_map_controller.dart';
export 'src/galileo_map_widget.dart';

Future<void> initGalileo() async {
  await rlib_gen.RustLib.init();
  simple.galileoFlutterInit(
        ffiPtr: ffi.NativeApi.initializeApiDLData.address,
      );
}
