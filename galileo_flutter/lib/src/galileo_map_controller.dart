import 'dart:async';

import 'package:flutter/foundation.dart' show kDebugMode, debugPrint;
import 'package:galileo_flutter/src/rust/api/dart_types.dart';
import 'package:galileo_flutter/src/rust/api/galileo_map.dart' as rlib;
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
  final RenderConfig config;
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
    RenderConfig? config,
    List<LayerConfig> layers = const [LayerConfig.osm()],
  }) async {
    try {
      // Get Flutter engine handle for texture registration
      final handle = await EngineContext.instance.getEngineHandle();

      // Create new session
      final sessionId = await rlib.createNewSession();

      // Use default config if none provided
      final renderConfig = config ?? await RenderConfig.default_();

      // Create the map instance
      final textureId = await rlib.createNewGalileoMap(
        sessionId: sessionId,
        engineHandle: handle,
        size: size,
        config: renderConfig,
      );

      // Create state broadcast
      final stateBroadcast = rx.BehaviorSubject<GalileoMapState>.seeded(
        GalileoMapState.initializing,
      );

      final controller = GalileoMapController._(
        size: size,
        config: renderConfig,
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

  /// Handle touch events from the map widget
  Future<void> handleTouchEvent({
    required double x,
    required double y,
    required TouchEventType eventType,
  }) async {
    if (!_running) return;

    try {
      // Forward touch event directly to session-based handler
      await rlib.handleSessionTouchEvent(
        sessionId: sessionId,
        event: TouchEvent(x: x, y: y, eventType: eventType),
      );
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error handling touch event: $e');
      }
    }
  }

  /// Handle scroll events from the map widget
  Future<void> handleScrollEvent({
    required double x,
    required double y,
    required double deltaX,
    required double deltaY,
  }) async {
    if (!_running) return;

    try {
      // Treat scroll events as zoom operations
      final zoomFactor = deltaY > 0 ? 0.9 : 1.1;
      await handleScaleEvent(
        focalX: x,
        focalY: y,
        scale: zoomFactor,
        rotation: 0.0,
        eventType: ScaleEventType.update,
      );
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error handling scroll event: $e');
      }
    }
  }

  /// Handle pan events from the map widget
  Future<void> handlePanEvent({
    required double x,
    required double y,
    required double deltaX,
    required double deltaY,
    required PanEventType eventType,
  }) async {
    if (!_running) return;

    try {
      await rlib.handleSessionPanEvent(
        sessionId: sessionId,
        event: PanEvent(
          x: x,
          y: y,
          deltaX: deltaX,
          deltaY: deltaY,
          eventType: eventType,
        ),
      );
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error handling pan event: $e');
      }
    }
  }

  /// Handle scale events from the map widget
  Future<void> handleScaleEvent({
    required double focalX,
    required double focalY,
    required double scale,
    required double rotation,
    required ScaleEventType eventType,
  }) async {
    if (!_running) return;

    try {
      await rlib.handleSessionScaleEvent(
        sessionId: sessionId,
        event: ScaleEvent(
          focalX: focalX,
          focalY: focalY,
          scale: scale,
          rotation: rotation,
          eventType: eventType,
        ),
      );
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Error handling scale event: $e');
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
