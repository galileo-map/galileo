import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'package:galileo_flutter/galileo_flutter.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Handle the key assertion error
  FlutterError.onError = (FlutterErrorDetails details) {
    if (details.exception is AssertionError &&
        details.exception.toString().contains('KeyDownEvent is dispatched')) {
      // Ignore this known Flutter issue in debug mode
      return;
    }
    FlutterError.dumpErrorToConsole(details);
  };

  // Initialize Galileo FFI with the real pointer
  await initGalileo();

  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Galileo Flutter Demo',
      theme: ThemeData(primarySwatch: Colors.blue),
      home: const GalileoMapPage(),
    );
  }
}

class GalileoMapPage extends StatefulWidget {
  const GalileoMapPage({super.key});

  @override
  State<GalileoMapPage> createState() => _GalileoMapPageState();
}

class _GalileoMapPageState extends State<GalileoMapPage> {
  MapViewport? currentViewport;
  String statusMessage = 'Loading...';
  String? _vectorTileStyle;

  @override
  void initState() {
    super.initState();
    _loadVectorTileStyle();
  }

  Future<void> _loadVectorTileStyle() async {
    try {
      final style = await rootBundle.loadString("assets/vt_style.json");
      setState(() {
        _vectorTileStyle = style;
        statusMessage = 'Map is ready';
      });
    } catch (e) {
      debugPrint('Error loading vector tile style: $e');
      setState(() {
        statusMessage = 'Error loading style: $e';
      });
    }
  }

  final style_str = '''{
      "rules": [
        {
          "symbol": {
            "line": {
              "stroke_color": "#000000ff",
              "width": 0.5
            }
          }
        },
        {
          "symbol": {
            "polygon": {
              "fill_color": "#999999ff"
            }
          }
        }
      ],
      "background": "#ffffffff"
    }''';

  void _onViewportChanged(MapViewport viewport) {
    // setState(() {
    //   currentViewport = viewport;
    //   statusMessage = "viewport: ${viewport.toString()}";
    // });
  }

  void _onMapTap(double x, double y) {
    // setState(() {
    //   statusMessage =
    //       'Tapped at: (${x.toStringAsFixed(1)}, ${y.toStringAsFixed(1)})';
    // });

    // // Clear the message after 2 seconds
    // Future.delayed(const Duration(seconds: 2), () {
    //   if (mounted) {
    //     setState(() {
    //       statusMessage =
    //           currentViewport != null
    //               ? "viewport: ${currentViewport}"
    //               : 'Map is ready';
    //     });
    //   }
    // });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Galileo Flutter Demo'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Column(
        children: [
          // Status bar
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(16.0),
            color: Colors.grey[100],
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Status: $statusMessage',
                  style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 4),
                const Text(
                  'Controls: Arrow keys to pan, +/- to zoom, mouse/touch to interact',
                  style: TextStyle(fontSize: 12, color: Colors.grey),
                ),
              ],
            ),
          ),
          // Map widget
          Expanded(
            child: Container(
              decoration: BoxDecoration(border: Border.all(color: Colors.grey)),
              child:
                  _vectorTileStyle == null
                      ? const Center(child: CircularProgressIndicator())
                      : GalileoMapWidget.fromConfig(
                        size: const MapSize(width: 800, height: 600),
                        layers: [
                          // const LayerConfig.osm(),
                          LayerConfig.vectorTiles(
                            // urlTemplate: 'https://api.protomaps.com/tiles/v4/{z}/{x}/{y}.mvt?key=07be0dc677fb29d3',
                            urlTemplate:
                                'https://api.maptiler.com/tiles/v3-openmaptiles/{z}/{x}/{y}.pbf?key=PZ3FHCeFcKn9AF7iL6SO',
                            styleJson: _vectorTileStyle!,
                          ),
                        ],
                        config: MapInitConfig(
                          backgroundColor: (0.1, 0.1, 0, 0.5),
                          enableMultisampling: true,
                          latlon: (0.0, 0.0),
                          mapSize: MapSize(width: 800, height: 600),
                          zoomLevel: 10,
                        ),
                        enableKeyboard: true,
                        onTap: _onMapTap,
                        onViewportChanged: _onViewportChanged,
                        child: Positioned(
                          top: 10,
                          right: 10,
                          child: Container(
                            padding: const EdgeInsets.all(8),
                            decoration: BoxDecoration(
                              color: Colors.white.withValues(alpha: 0.9),
                              borderRadius: BorderRadius.circular(8),
                              boxShadow: [
                                BoxShadow(
                                  color: Colors.black.withValues(alpha: 0.1),
                                  blurRadius: 4,
                                  offset: const Offset(0, 2),
                                ),
                              ],
                            ),
                            child: const Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Text(
                                  'Map Controls:',
                                  style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
                                ),
                                SizedBox(height: 4),
                                Text('• Drag to pan', style: TextStyle(fontSize: 10)),
                                Text('• Pinch to zoom', style: TextStyle(fontSize: 10)),
                                Text('• Arrow keys to pan', style: TextStyle(fontSize: 10)),
                                Text('• +/- to zoom', style: TextStyle(fontSize: 10)),
                              ],
                            ),
                          ),
                        ),
                      ),
            ),
          ),
          // Control panel
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(16.0),
            color: Colors.grey[50],
            child: Wrap(
              spacing: 8.0,
              runSpacing: 8.0,
              children: [
                ElevatedButton.icon(
                  onPressed: () async {
                    // TODO: Implement in Phase 2
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('Zoom to location (coming in Phase 2)')),
                    );
                  },
                  icon: const Icon(Icons.my_location),
                  label: const Text('My Location'),
                ),
                ElevatedButton.icon(
                  onPressed: () async {
                    // TODO: Implement in Phase 2
                    ScaffoldMessenger.of(
                      context,
                    ).showSnackBar(const SnackBar(content: Text('Reset view (coming in Phase 2)')));
                  },
                  icon: const Icon(Icons.refresh),
                  label: const Text('Reset View'),
                ),
                ElevatedButton.icon(
                  onPressed: () async {
                    // TODO: Implement in Phase 2
                    ScaffoldMessenger.of(
                      context,
                    ).showSnackBar(const SnackBar(content: Text('Add layer (coming in Phase 2)')));
                  },
                  icon: const Icon(Icons.layers),
                  label: const Text('Add Layer'),
                ),
              ],
            ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          showDialog(
            context: context,
            builder:
                (context) => AlertDialog(
                  title: const Text('About'),
                  content: const Text('This is a Galileo Flutter integration demo.\n'),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.of(context).pop(),
                      child: const Text('OK'),
                    ),
                  ],
                ),
          );
        },
        child: const Icon(Icons.info),
      ),
    );
  }
}
