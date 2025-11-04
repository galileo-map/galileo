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

  const GalileoMapWidget._({
    super.key,
    required this.controller,
    this.child,
    this.autoDispose = true,
    this.enableKeyboard = true,
    this.focusNode,
    this.onTap,
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
  }) {
    return GalileoMapWidget._(
      key: key,
      controller: controller,
      autoDispose: autoDispose,
      enableKeyboard: enableKeyboard,
      focusNode: focusNode,
      onTap: onTap,
      child: child,
    );
  }

  /// Create a GalileoMapWidget with configuration
  static Widget fromConfig({
    Key? key,
    required MapSize size,
    required MapInitConfig config,
    List<LayerConfig> layers = const [LayerConfig.osm()],
    bool autoDispose = true,
    bool enableKeyboard = true,
    FocusNode? focusNode,
    Widget? child,
    void Function(double x, double y)? onTap,
    void Function(MapViewport viewport)? onViewportChanged,
  }) {
    return FutureBuilder(
      future: GalileoMapController.create(size: size, config: config, layers: layers),
      builder: (ctx, res) {
        if (res.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }

        if (res.hasError) {
          return Center(
            child: Text('Error: ${res.error}', style: const TextStyle(color: Colors.red)),
          );
        }

        final (controller, err) = res.data!;
        if (err != null) {
          return Center(child: Text('Error: $err', style: const TextStyle(color: Colors.red)));
        }

        return GalileoMapWidget.fromController(
          key: key,
          controller: controller!,
          autoDispose: autoDispose,
          enableKeyboard: enableKeyboard,
          focusNode: focusNode,
          onTap: onTap,
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
  Set<LogicalKeyboardKey> _pressedKeys = {};

  Offset? _lastPointerPosition;
  MapSize? _lastMapSize;
  double _lastPinchScaleValue = 1;
  bool _isPinchScaling = false;

  @override
  void initState() {
    super.initState();

    _focusNode = widget.focusNode ?? FocusNode();

    if (widget.enableKeyboard) {
      HardwareKeyboard.instance.addHandler(_handleKeyEvent);
    }

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
          Text(message, style: const TextStyle(fontSize: 16), textAlign: TextAlign.center),
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

    // Wrap with low-level pointer events for more control
    mapContent = Listener(
      behavior: HitTestBehavior.opaque,
      onPointerDown: (event) {
        if (_isPinchScaling) {
          return;
        }
        
        // Request focus for keyboard events
        if (widget.enableKeyboard) {
          _focusNode.requestFocus();
        }
        widget.onTap?.call(event.localPosition.dx, event.localPosition.dy);
        _lastPointerPosition = event.localPosition;

        // Handle button press for primary pointer
        final mouseEvent = UserEvent.buttonPressed(
          MouseButton.left, // Default to left for touch
          MouseEvent(
            screenPointerPosition: Point2(x: event.localPosition.dx, y: event.localPosition.dy),
            buttons: const MouseButtonsState(
              left: MouseButtonState.pressed,
              middle: MouseButtonState.released,
              right: MouseButtonState.released,
            ),
          ),
        );
        widget.controller.handleEvent(mouseEvent);
      },
      onPointerUp: (event) {
        _lastPointerPosition = null;
        // Handle button release for primary pointer
        final mouseEvent = UserEvent.buttonReleased(
          MouseButton.left, // Default to left for touch
          MouseEvent(
            screenPointerPosition: Point2(x: event.localPosition.dx, y: event.localPosition.dy),
            buttons: const MouseButtonsState(
              left: MouseButtonState.released,
              middle: MouseButtonState.released,
              right: MouseButtonState.released,
            ),
          ),
        );
        widget.controller.handleEvent(mouseEvent);
      },
      onPointerCancel: (event) {
        _lastPointerPosition = null;
        // Release button on cancel
        final mouseEvent = UserEvent.buttonReleased(
          MouseButton.left,
          MouseEvent(
            screenPointerPosition: Point2(x: event.localPosition.dx, y: event.localPosition.dy),
            buttons: const MouseButtonsState(
              left: MouseButtonState.released,
              middle: MouseButtonState.released,
              right: MouseButtonState.released,
            ),
          ),
        );
        widget.controller.handleEvent(mouseEvent);
      },
      onPointerSignal: (event) {
        if (event is PointerScrollEvent) {
          // Handle scroll as zoom event
          final zoomFactor = event.scrollDelta.dy > 0 ? -1.0 : 1.0;
          final scrollEvent = UserEvent.scroll(
            zoomFactor,
            MouseEvent(
              screenPointerPosition: Point2(x: event.localPosition.dx, y: event.localPosition.dy),
              buttons: const MouseButtonsState(
                left: MouseButtonState.released,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(scrollEvent);
        }
      },
      onPointerMove: (event) {
        if (_isPinchScaling) {
          return;
        }
        
        if (event.buttons == 0) {
          return;
        }

        final currentPosition = event.localPosition;

        if (_lastPointerPosition case final lastPosition?) {
          final delta = currentPosition - lastPosition;

          if (delta.dx.abs() > 0.1 || delta.dy.abs() > 0.1) {
            final panEvent = UserEvent.drag(
              MouseButton.left,
              Vector2(dx: delta.dx, dy: delta.dy),
              MouseEvent(
                screenPointerPosition: Point2(x: currentPosition.dx, y: currentPosition.dy),
                buttons: const MouseButtonsState(
                  left: MouseButtonState.pressed,
                  middle: MouseButtonState.released,
                  right: MouseButtonState.released,
                ),
              ),
            );
            widget.controller.handleEvent(panEvent);
          }
        }

        _lastPointerPosition = currentPosition;
      },

      child: mapContent,
    );

    // Add keyboard support if enabled
    if (widget.enableKeyboard) {
      mapContent = Focus(focusNode: _focusNode, autofocus: true, child: mapContent);
    }

    // Add pinch-to-zoom support
    mapContent = GestureDetector(
      onScaleStart: (details) {
        _lastPinchScaleValue = 1.0;
      },
      onScaleUpdate: (details) {
        if (details.pointerCount >= 2) {
          if (!_isPinchScaling) {
            _isPinchScaling = true;
            _lastPinchScaleValue = details.scale;
            return;
          }
          
          if (details.scale != _lastPinchScaleValue) {
            // TODO: test this
            final scaleDelta = details.scale / _lastPinchScaleValue;
            final zoomEvent = UserEvent.zoom(
              1.0 / scaleDelta,
              Point2(x: details.localFocalPoint.dx, y: details.localFocalPoint.dy),
            );
            widget.controller.handleEvent(zoomEvent);
            
            _lastPinchScaleValue = details.scale;
          }
        }
      },
      onScaleEnd: (details) {
        _lastPinchScaleValue = 1.0;
        _isPinchScaling = false;
      },
      child: mapContent,
    );

    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(constraints.maxWidth, constraints.maxHeight);
        final newMapSize = MapSize(width: size.width.toInt(), height: size.height.toInt());
        
        if (_lastMapSize == null || 
            _lastMapSize!.width != newMapSize.width || 
            _lastMapSize!.height != newMapSize.height) {
          _lastMapSize = newMapSize;
          // resize in next frame
          // TODO: test this
          Future.microtask(() => widget.controller.resize(newMapSize));
        }
        
        return mapContent;
      },
    );
  }

  bool _handleKeyEvent(KeyEvent event) {
    // Handle keyboard events for map navigation only if focused
    if (!_focusNode.hasFocus) return false;

    if (event is KeyDownEvent && !_pressedKeys.contains(event.logicalKey)) {
      _pressedKeys.add(event.logicalKey);
      switch (event.logicalKey) {
        case LogicalKeyboardKey.arrowUp:
          // Pan up using drag event
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.drag(
            MouseButton.left,
            const Vector2(dx: 0, dy: 20),
            MouseEvent(
              screenPointerPosition: Point2(x: centerX, y: centerY),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.arrowDown:
          // Pan down using drag event
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.drag(
            MouseButton.left,
            const Vector2(dx: 0, dy: -20),
            MouseEvent(
              screenPointerPosition: Point2(x: centerX, y: centerY),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.arrowLeft:
          // Pan left using drag event
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.drag(
            MouseButton.left,
            const Vector2(dx: 20, dy: 0),
            MouseEvent(
              screenPointerPosition: Point2(x: centerX, y: centerY),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.arrowRight:
          // Pan right using drag event
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.drag(
            MouseButton.left,
            const Vector2(dx: -20, dy: 0),
            MouseEvent(
              screenPointerPosition: Point2(x: centerX, y: centerY),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.equal:
        case LogicalKeyboardKey.numpadAdd:
          // Zoom in
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.zoom(0.9, Point2(x: centerX, y: centerY));
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.minus:
        case LogicalKeyboardKey.numpadSubtract:
          // Zoom out
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.zoom(1.1, Point2(x: centerX, y: centerY));
          widget.controller.handleEvent(userEvent);
          break;
      }
    } else if (event is KeyRepeatEvent && _pressedKeys.contains(event.logicalKey)) {
      // Handle repeat events for continuous panning/zooming
      switch (event.logicalKey) {
        case LogicalKeyboardKey.arrowUp:
          // Pan up using drag event
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.drag(
            MouseButton.left,
            const Vector2(dx: 0, dy: 20),
            MouseEvent(
              screenPointerPosition: Point2(x: centerX, y: centerY),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.arrowDown:
          // Pan down using drag event
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.drag(
            MouseButton.left,
            const Vector2(dx: 0, dy: -20),
            MouseEvent(
              screenPointerPosition: Point2(x: centerX, y: centerY),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.arrowLeft:
          // Pan left using drag event
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.drag(
            MouseButton.left,
            const Vector2(dx: 20, dy: 0),
            MouseEvent(
              screenPointerPosition: Point2(x: centerX, y: centerY),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.arrowRight:
          // Pan right using drag event
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.drag(
            MouseButton.left,
            const Vector2(dx: -20, dy: 0),
            MouseEvent(
              screenPointerPosition: Point2(x: centerX, y: centerY),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.equal:
        case LogicalKeyboardKey.numpadAdd:
          // Zoom in
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.zoom(0.9, Point2(x: centerX, y: centerY));
          widget.controller.handleEvent(userEvent);
          break;
        case LogicalKeyboardKey.minus:
        case LogicalKeyboardKey.numpadSubtract:
          // Zoom out
          final centerX = widget.controller.size.width / 2;
          final centerY = widget.controller.size.height / 2;
          final userEvent = UserEvent.zoom(1.1, Point2(x: centerX, y: centerY));
          widget.controller.handleEvent(userEvent);
          break;
      }
    } else if (event is KeyUpEvent && _pressedKeys.contains(event.logicalKey)) {
      _pressedKeys.remove(event.logicalKey);
    }
    return false;
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
          // Future.microtask(() async => await widget.controller.requestRedraw());
          return _buildMapWidget(textureId);
        } else {
          return _buildLoadingWidget('Preparing texture...');
        }

      case GalileoMapState.stopped:
        return const Center(child: Text('Map stopped', style: TextStyle(fontSize: 16)));
    }
  }

  @override
  void dispose() {
    if (widget.enableKeyboard) {
      HardwareKeyboard.instance.removeHandler(_handleKeyEvent);
    }

    super.dispose();

    Future.microtask(() async {
      streamSubscription?.cancel();
      if (widget.autoDispose) {
        try {
          if (kDebugMode) {
            debugPrint('Disposing Galileo map controller (${widget.controller.sessionId})');
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
