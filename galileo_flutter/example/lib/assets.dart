import 'package:flutter/services.dart';

/// Helper class for loading assets with typed paths
class Assets {
  Assets._(); // Private constructor to prevent instantiation

  // Asset paths
  static const String vectorTileStyle = 'assets/vt_style.json';

  // Helper methods
  static Future<String> loadString(String path) => rootBundle.loadString(path);
  
  static Future<String> loadVectorTileStyle() => loadString(vectorTileStyle);
}

