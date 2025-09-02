import 'dart:async';

import 'package:flutter/foundation.dart' show kDebugMode, debugPrint;
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:galileo_flutter/src/galileo_map_controller.dart';
import 'package:galileo_flutter/src/rust/api/dart_types.dart';

/// A widget that displays a Galileo map with interactive controls
class GalileoMapWidget extends StatefulWidget {
  final GalileoMapController controller;
  final Widget? child;

  /// Whether to dispose the controller when the widget disposes
  final bool autoDispose;

  /// Whether to enable keyboard input
  final bool enableKeyboard;

  /// Focus node for keyboard events
  final FocusNode? focusNode;

  /// Called when the map is tapped
  final void Function(double x, double y)? onTap;

  /// Called when the map viewport changes
  final void Function(MapViewport viewport)? onViewportChanged;

  const GalileoMapWidget._({
    super.key,
    required this.controller,
    this.child,
    this.autoDispose = true,
    this.enableKeyboard = true,
    this.focusNode,
    this.onTap,
    this.onViewportChanged,
  });

  /// Create a GalileoMapWidget from an existing controller
  factory GalileoMapWidget.fromController({
    Key? key,
    required GalileoMapController controller,
    bool autoDispose = true,
    bool enableKeyboard = true,
    FocusNode? focusNode,
    Widget? child,
    void Function(double x, double y)? onTap,
    void Function(MapViewport viewport)? onViewportChanged,
  }) {
    return GalileoMapWidget._(
      key: key,
      controller: controller,
      autoDispose: autoDispose,
      enableKeyboard: enableKeyboard,
      focusNode: focusNode,
      onTap: onTap,
      onViewportChanged: onViewportChanged,
      child: child,
    );
  }

  /// Create a GalileoMapWidget with configuration
  static Widget fromConfig({
    Key? key,
    required MapSize size,
    RenderConfig? config,
    List<LayerConfig> layers = const [LayerConfig.osm()],
    bool autoDispose = true,
    bool enableKeyboard = true,
    FocusNode? focusNode,
    Widget? child,
    void Function(double x, double y)? onTap,
    void Function(MapViewport viewport)? onViewportChanged,
  }) {
    return FutureBuilder(
      future: GalileoMapController.create(
        size: size,
        config: config,
        layers: layers,
      ),
      builder: (ctx, res) {
        if (res.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }

        if (res.hasError) {
          return Center(
            child: Text(
              'Error: ${res.error}',
              style: const TextStyle(color: Colors.red),
            ),
          );
        }

        final (controller, err) = res.data!;
        if (err != null) {
          return Center(
            child: Text(
              'Error: $err',
              style: const TextStyle(color: Colors.red),
            ),
          );
        }

        return GalileoMapWidget.fromController(
          key: key,
          controller: controller!,
          autoDispose: autoDispose,
          enableKeyboard: enableKeyboard,
          focusNode: focusNode,
          onTap: onTap,
          onViewportChanged: onViewportChanged,
          child: child,
        );
      },
    );
  }

  @override
  State<GalileoMapWidget> createState() => _GalileoMapWidgetState();
}

class _GalileoMapWidgetState extends State<GalileoMapWidget> {
  GalileoMapState? currentState;
  StreamSubscription<GalileoMapState>? streamSubscription;
  late FocusNode _focusNode;
  bool _isDragging = false;
  Offset? _lastPanPosition;

  @override
  void initState() {
    super.initState();

    _focusNode = widget.focusNode ?? FocusNode();

    streamSubscription = widget.controller.stateStream.listen((state) {
      if (mounted) {
        setState(() {
          currentState = state;
        });
      }
    });
  }

  Widget _buildLoadingWidget(String message) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const CircularProgressIndicator(),
          const SizedBox(height: 16),
          Text(
            message,
            style: const TextStyle(fontSize: 16),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  Widget _buildErrorWidget(String message) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.error_outline, color: Colors.red, size: 48),
          const SizedBox(height: 16),
          Text(
            'Error: $message',
            style: const TextStyle(color: Colors.red, fontSize: 16),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () {
              // TODO: Implement retry logic in Phase 2
            },
            child: const Text('Retry'),
          ),
        ],
      ),
    );
  }

  Widget _buildMapWidget(int textureId) {
    Widget mapContent = Stack(
      children: [
        // The actual map texture
        Positioned.fill(child: Texture(textureId: textureId)),
        // Optional child widget overlay
        if (widget.child != null) widget.child!,
      ],
    );

    // Wrap with gesture detection
    mapContent = GestureDetector(
      onTap: () {
        // Request focus for keyboard events
        if (widget.enableKeyboard) {
          _focusNode.requestFocus();
        }
      },
      onTapDown: (details) {
        final localPosition = details.localPosition;
        widget.onTap?.call(localPosition.dx, localPosition.dy);

        // Handle touch down event
        widget.controller.handleTouchEvent(
          x: localPosition.dx,
          y: localPosition.dy,
          eventType: TouchEventType.down,
        );
      },
      onTapUp: (details) {
        final localPosition = details.localPosition;

        // Handle touch up event
        widget.controller.handleTouchEvent(
          x: localPosition.dx,
          y: localPosition.dy,
          eventType: TouchEventType.up,
        );
      },
      onPanStart: (details) {
        _isDragging = true;
        _lastPanPosition = details.localPosition;

        widget.controller.handlePanEvent(
          x: details.localPosition.dx,
          y: details.localPosition.dy,
          deltaX: 0,
          deltaY: 0,
          eventType: PanEventType.start,
        );
      },
      onPanUpdate: (details) {
        if (_isDragging && _lastPanPosition != null) {
          final currentPosition = details.localPosition;
          final delta = currentPosition - _lastPanPosition!;

          widget.controller.handlePanEvent(
            x: currentPosition.dx,
            y: currentPosition.dy,
            deltaX: delta.dx,
            deltaY: delta.dy,
            eventType: PanEventType.update,
          );

          _lastPanPosition = currentPosition;
        }
      },
      onPanEnd: (details) {
        if (_isDragging && _lastPanPosition != null) {
          widget.controller.handlePanEvent(
            x: _lastPanPosition!.dx,
            y: _lastPanPosition!.dy,
            deltaX: 0,
            deltaY: 0,
            eventType: PanEventType.end,
          );
        }

        _isDragging = false;
        _lastPanPosition = null;
      },
      onScaleStart: (details) {
        widget.controller.handleScaleEvent(
          focalX: details.focalPoint.dx,
          focalY: details.focalPoint.dy,
          scale: 1.0,
          rotation: 0.0,
          eventType: ScaleEventType.start,
        );
      },
      onScaleUpdate: (details) {
        widget.controller.handleScaleEvent(
          focalX: details.focalPoint.dx,
          focalY: details.focalPoint.dy,
          scale: details.scale,
          rotation: details.rotation,
          eventType: ScaleEventType.update,
        );
      },
      onScaleEnd: (details) {
        widget.controller.handleScaleEvent(
          focalX: 0.0,
          focalY: 0.0,
          scale: 1.0,
          rotation: 0.0,
          eventType: ScaleEventType.end,
        );
      },
      child: mapContent,
    );

    // Wrap with low-level pointer events for more control
    mapContent = Listener(
      onPointerDown: (event) {
        widget.controller.handleTouchEvent(
          x: event.localPosition.dx,
          y: event.localPosition.dy,
          eventType: TouchEventType.down,
        );
      },
      onPointerMove: (event) {
        widget.controller.handleTouchEvent(
          x: event.localPosition.dx,
          y: event.localPosition.dy,
          eventType: TouchEventType.move,
        );
      },
      onPointerUp: (event) {
        widget.controller.handleTouchEvent(
          x: event.localPosition.dx,
          y: event.localPosition.dy,
          eventType: TouchEventType.up,
        );
      },
      onPointerCancel: (event) {
        widget.controller.handleTouchEvent(
          x: event.localPosition.dx,
          y: event.localPosition.dy,
          eventType: TouchEventType.cancel,
        );
      },
      onPointerSignal: (event) {
        if (event is PointerScrollEvent) {
          widget.controller.handleScrollEvent(
            x: event.localPosition.dx,
            y: event.localPosition.dy,
            deltaX: event.scrollDelta.dx,
            deltaY: event.scrollDelta.dy,
          );
        }
      },
      child: mapContent,
    );

    // Add keyboard support if enabled
    if (widget.enableKeyboard) {
      mapContent = KeyboardListener(
        focusNode: _focusNode,
        autofocus: true,
        onKeyEvent: (event) {
          _handleKeyEvent(event);
        },
        child: mapContent,
      );
    }

    return mapContent;
  }

  void _handleKeyEvent(KeyEvent event) {
    // Handle keyboard events for map navigation
    if (event is KeyDownEvent) {
      switch (event.logicalKey) {
        case LogicalKeyboardKey.arrowUp:
          // Pan up
          widget.controller.handlePanEvent(
            x: widget.controller.size.width / 2,
            y: widget.controller.size.height / 2,
            deltaX: 0,
            deltaY: -20,
            eventType: PanEventType.update,
          );
          break;
        case LogicalKeyboardKey.arrowDown:
          // Pan down
          widget.controller.handlePanEvent(
            x: widget.controller.size.width / 2,
            y: widget.controller.size.height / 2,
            deltaX: 0,
            deltaY: 20,
            eventType: PanEventType.update,
          );
          break;
        case LogicalKeyboardKey.arrowLeft:
          // Pan left
          widget.controller.handlePanEvent(
            x: widget.controller.size.width / 2,
            y: widget.controller.size.height / 2,
            deltaX: -20,
            deltaY: 0,
            eventType: PanEventType.update,
          );
          break;
        case LogicalKeyboardKey.arrowRight:
          // Pan right
          widget.controller.handlePanEvent(
            x: widget.controller.size.width / 2,
            y: widget.controller.size.height / 2,
            deltaX: 20,
            deltaY: 0,
            eventType: PanEventType.update,
          );
          break;
        case LogicalKeyboardKey.equal:
        case LogicalKeyboardKey.numpadAdd:
          // Zoom in
          widget.controller.handleScaleEvent(
            focalX: widget.controller.size.width / 2,
            focalY: widget.controller.size.height / 2,
            scale: 1.1,
            rotation: 0.0,
            eventType: ScaleEventType.update,
          );
          break;
        case LogicalKeyboardKey.minus:
        case LogicalKeyboardKey.numpadSubtract:
          // Zoom out
          widget.controller.handleScaleEvent(
            focalX: widget.controller.size.width / 2,
            focalY: widget.controller.size.height / 2,
            scale: 0.9,
            rotation: 0.0,
            eventType: ScaleEventType.update,
          );
          break;
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (currentState == null) {
      return _buildLoadingWidget('Initializing map...');
    }

    switch (currentState!) {
      case GalileoMapState.initializing:
        return _buildLoadingWidget('Initializing Galileo map...');

      case GalileoMapState.error:
        return _buildErrorWidget('Map encountered an error');

      case GalileoMapState.ready:
        final textureId = widget.controller.textureId;
        if (textureId != null) {
          return _buildMapWidget(textureId);
        } else {
          return _buildLoadingWidget('Preparing texture...');
        }

      case GalileoMapState.stopped:
        return const Center(
          child: Text('Map stopped', style: TextStyle(fontSize: 16)),
        );
    }
  }

  @override
  void dispose() {
    super.dispose();

    Future.microtask(() async {
      streamSubscription?.cancel();
      if (widget.autoDispose) {
        try {
          if (kDebugMode) {
            debugPrint(
              'Disposing Galileo map controller (${widget.controller.sessionId})',
            );
          }
          await widget.controller.dispose();
        } catch (e) {
          if (kDebugMode) {
            debugPrint('Error disposing Galileo map controller: $e');
          }
        }
      }
    });

    // Dispose focus node if we created it
    if (widget.focusNode == null) {
      _focusNode.dispose();
    }
  }
}
