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

  // Coordinate scaling factors
  double _scaleX = 1.0;
  double _scaleY = 1.0;
  Size? _lastConstraintSize;

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

  void _updateScaleFactors(Size constraintSize) {
    if (_lastConstraintSize != constraintSize) {
      _lastConstraintSize = constraintSize;

      _scaleX = widget.controller.size.width / constraintSize.width;
      _scaleY = widget.controller.size.height / constraintSize.height;
    }
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
      onScaleUpdate: (details) {
        final scaledDeltaX = details.focalPointDelta.dx * _scaleX;
        final scaledDeltaY = details.focalPointDelta.dy * _scaleY;

        if (details.focalPointDelta.dx.abs() > 0.1 || details.focalPointDelta.dy.abs() > 0.1) {
          final panEvent = UserEvent.drag(
            MouseButton.left,
            Vector2(dx: scaledDeltaX, dy: scaledDeltaY),
            MouseEvent(
              screenPointerPosition: Point2(x: details.focalPoint.dx, y: details.focalPoint.dy),
              buttons: const MouseButtonsState(
                left: MouseButtonState.pressed,
                middle: MouseButtonState.released,
                right: MouseButtonState.released,
              ),
            ),
          );
          widget.controller.handleEvent(panEvent);
        }
      },
      // onScaleEnd: (details) {
      //   // No specific end event needed for zoom
      // },
      child: mapContent,
    );

    // Wrap with low-level pointer events for more control
    mapContent = Listener(
      onPointerDown: (event) {
        // Request focus for keyboard events
        if (widget.enableKeyboard) {
          _focusNode.requestFocus();
        }

        widget.onTap?.call(event.localPosition.dx, event.localPosition.dy);

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
      onPointerMove: (event) {
        // Handle single pointer move
        final mouseEvent = UserEvent.pointerMoved(
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
          final mapX = event.localPosition.dx * _scaleX;
          final mapY = event.localPosition.dy * _scaleY;

          final zoomFactor = event.scrollDelta.dy > 0 ? -1.0 : 1.0;
          final scrollEvent = UserEvent.scroll(
            zoomFactor,
            MouseEvent(
              screenPointerPosition: Point2(x: mapX, y: mapY),
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
      child: mapContent,
    );

    // Add keyboard support if enabled
    if (widget.enableKeyboard) {
      mapContent = Focus(focusNode: _focusNode, autofocus: true, child: mapContent);
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(constraints.maxWidth, constraints.maxHeight);
        _updateScaleFactors(size);
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
          Future.microtask(() async => await widget.controller.requestRedraw());
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
