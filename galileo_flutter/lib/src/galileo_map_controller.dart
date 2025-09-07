import 'dart:async';

import 'package:flutter/foundation.dart' show kDebugMode, debugPrint;
import 'package:galileo_flutter/src/rust/api/dart_types.dart';
import 'package:galileo_flutter/src/rust/api/api.dart' as rlib;

import 'package:irondash_engine_context/irondash_engine_context.dart';
import "package:rxdart/rxdart.dart" as rx;

/// State of a Galileo map instance
enum GalileoMapState {
  /// Map is being initialized
  initializing,

  /// Map is ready and rendering
  ready,

  /// Map encountered an error
  error,

  /// Map has been stopped/destroyed
  stopped,
}

/// Controller for managing a Galileo map instance
class GalileoMapController {
  final MapSize size;
  final MapInitConfig config;
  final List<LayerConfig> layers;

  final int sessionId;
  final rx.BehaviorSubject<GalileoMapState> _stateBroadcast;
  final StreamSubscription<GalileoMapState>? _originalSub;
  bool _running = false;
  int? _textureId;

  GalileoMapController._({
    required this.size,
    required this.config,
    required this.layers,
    required this.sessionId,
    required rx.BehaviorSubject<GalileoMapState> stateBroadcast,
    StreamSubscription<GalileoMapState>? originalSub,
  }) : _stateBroadcast = stateBroadcast,
       _originalSub = originalSub;

  /// Stream of map state changes
  Stream<GalileoMapState> get stateStream => _stateBroadcast.stream;

  /// Current map state
  GalileoMapState get currentState => _stateBroadcast.value;

  /// Texture ID for rendering (null if not ready)
  int? get textureId => _textureId;

  /// Whether the map is currently running
  bool get isRunning => _running;

  /// Create a new Galileo map controller
  static Future<(GalileoMapController?, String?)> create({
    required MapSize size,
    MapInitConfig? config,
    List<LayerConfig> layers = const [LayerConfig.osm()],
  }) async {
    try {
      // Get Flutter engine handle for texture registration
      final handle = await EngineContext.instance.getEngineHandle();

      // Create new session
      final sessionId = await rlib.createNewSession();

      // Use default config if none provided
      final MapInitConfig = config ?? await MapInitConfig.default_();

      // Create the map instance
      final textureId = await rlib.createNewGalileoMap(
        sessionId: sessionId,
        engineHandle: handle,
        size: size,
        config: MapInitConfig,
      );

      // Create state broadcast
      final stateBroadcast = rx.BehaviorSubject<GalileoMapState>.seeded(
        GalileoMapState.initializing,
      );

      final controller = GalileoMapController._(
        size: size,
        config: MapInitConfig,
        layers: layers,
        sessionId: sessionId,
        stateBroadcast: stateBroadcast,
        originalSub: null,
      );

      controller._textureId = textureId;
      controller._running = true;

      // Start session keep-alive task
      controller._startKeepAliveTask();

      // Set state to ready
      controller._stateBroadcast.add(GalileoMapState.ready);

      return (controller, null);
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error creating Galileo map: $e');
      }
      return (null, e.toString());
    }
  }

  /// Start the session keep-alive task
  void _startKeepAliveTask() {
    Future.microtask(() async {
      while (_running) {
        try {
          // Ping Rust side to announce we still want the stream
          await rlib.markSessionAlive(sessionId: sessionId);
          await Future.delayed(const Duration(seconds: 1));
        } catch (e) {
          if (kDebugMode) {
            debugPrint('Error in keep-alive task: $e');
          }
          if (_running) {
            _stateBroadcast.add(GalileoMapState.error);
          }
          break;
        }
      }
    });
  }

  /// Handle user events from the map widget
  Future<void> handleEvent(UserEvent event) async {
    if (!_running) return;

    try {
      await rlib.handleEventForSession(sessionId: sessionId, event: event);
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error handling event: $e');
      }
    }
  }

  /// Get the current map viewport
  Future<MapViewport?> getViewport() async {
    if (!_running) return null;

    try {
      return await rlib.getSessionViewport(sessionId: sessionId);
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error getting viewport: $e');
      }
      return null;
    }
  }

  /// Set the map viewport
  Future<void> setViewport(MapViewport viewport) async {
    if (!_running) return;

    try {
      await rlib.setSessionViewport(sessionId: sessionId, viewport: viewport);
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error setting viewport: $e');
      }
    }
  }

  /// Add a layer to the map
  Future<void> addLayer(LayerConfig layer) async {
    if (!_running) return;

    try {
      await rlib.addSessionLayer(sessionId: sessionId, layerConfig: layer);
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error adding layer: $e');
      }
    }
  }

  /// Resize the map
  Future<void> resize(MapSize newSize) async {
    if (!_running) return;

    try {
      await rlib.resizeSessionSize(sessionId: sessionId, size: newSize);
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error resizing map: $e');
      }
    }
  }

  /// Dispose of the controller and clean up resources
  Future<void> dispose() async {
    _running = false;

    try {
      await rlib.destroySession(sessionId: sessionId);
      await _originalSub?.cancel();
      await _stateBroadcast.close();
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error disposing Galileo map controller: $e');
      }
    }
  }
}
