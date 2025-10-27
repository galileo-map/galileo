// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dart_types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

/// @nodoc
mixin _$LayerConfig {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() osm,
    required TResult Function(String urlTemplate, String? attribution)
    rasterTiles,
    required TResult Function(
      String urlTemplate,
      String styleJson,
      String? attribution,
    )
    vectorTiles,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? osm,
    TResult? Function(String urlTemplate, String? attribution)? rasterTiles,
    TResult? Function(
      String urlTemplate,
      String styleJson,
      String? attribution,
    )?
    vectorTiles,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? osm,
    TResult Function(String urlTemplate, String? attribution)? rasterTiles,
    TResult Function(String urlTemplate, String styleJson, String? attribution)?
    vectorTiles,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(LayerConfig_Osm value) osm,
    required TResult Function(LayerConfig_RasterTiles value) rasterTiles,
    required TResult Function(LayerConfig_VectorTiles value) vectorTiles,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LayerConfig_Osm value)? osm,
    TResult? Function(LayerConfig_RasterTiles value)? rasterTiles,
    TResult? Function(LayerConfig_VectorTiles value)? vectorTiles,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LayerConfig_Osm value)? osm,
    TResult Function(LayerConfig_RasterTiles value)? rasterTiles,
    TResult Function(LayerConfig_VectorTiles value)? vectorTiles,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $LayerConfigCopyWith<$Res> {
  factory $LayerConfigCopyWith(
    LayerConfig value,
    $Res Function(LayerConfig) then,
  ) = _$LayerConfigCopyWithImpl<$Res, LayerConfig>;
}

/// @nodoc
class _$LayerConfigCopyWithImpl<$Res, $Val extends LayerConfig>
    implements $LayerConfigCopyWith<$Res> {
  _$LayerConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of LayerConfig
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$LayerConfig_OsmImplCopyWith<$Res> {
  factory _$$LayerConfig_OsmImplCopyWith(
    _$LayerConfig_OsmImpl value,
    $Res Function(_$LayerConfig_OsmImpl) then,
  ) = __$$LayerConfig_OsmImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$LayerConfig_OsmImplCopyWithImpl<$Res>
    extends _$LayerConfigCopyWithImpl<$Res, _$LayerConfig_OsmImpl>
    implements _$$LayerConfig_OsmImplCopyWith<$Res> {
  __$$LayerConfig_OsmImplCopyWithImpl(
    _$LayerConfig_OsmImpl _value,
    $Res Function(_$LayerConfig_OsmImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of LayerConfig
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$LayerConfig_OsmImpl extends LayerConfig_Osm {
  const _$LayerConfig_OsmImpl() : super._();

  @override
  String toString() {
    return 'LayerConfig.osm()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is _$LayerConfig_OsmImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() osm,
    required TResult Function(String urlTemplate, String? attribution)
    rasterTiles,
    required TResult Function(
      String urlTemplate,
      String styleJson,
      String? attribution,
    )
    vectorTiles,
  }) {
    return osm();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? osm,
    TResult? Function(String urlTemplate, String? attribution)? rasterTiles,
    TResult? Function(
      String urlTemplate,
      String styleJson,
      String? attribution,
    )?
    vectorTiles,
  }) {
    return osm?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? osm,
    TResult Function(String urlTemplate, String? attribution)? rasterTiles,
    TResult Function(String urlTemplate, String styleJson, String? attribution)?
    vectorTiles,
    required TResult orElse(),
  }) {
    if (osm != null) {
      return osm();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(LayerConfig_Osm value) osm,
    required TResult Function(LayerConfig_RasterTiles value) rasterTiles,
    required TResult Function(LayerConfig_VectorTiles value) vectorTiles,
  }) {
    return osm(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LayerConfig_Osm value)? osm,
    TResult? Function(LayerConfig_RasterTiles value)? rasterTiles,
    TResult? Function(LayerConfig_VectorTiles value)? vectorTiles,
  }) {
    return osm?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LayerConfig_Osm value)? osm,
    TResult Function(LayerConfig_RasterTiles value)? rasterTiles,
    TResult Function(LayerConfig_VectorTiles value)? vectorTiles,
    required TResult orElse(),
  }) {
    if (osm != null) {
      return osm(this);
    }
    return orElse();
  }
}

abstract class LayerConfig_Osm extends LayerConfig {
  const factory LayerConfig_Osm() = _$LayerConfig_OsmImpl;
  const LayerConfig_Osm._() : super._();
}

/// @nodoc
abstract class _$$LayerConfig_RasterTilesImplCopyWith<$Res> {
  factory _$$LayerConfig_RasterTilesImplCopyWith(
    _$LayerConfig_RasterTilesImpl value,
    $Res Function(_$LayerConfig_RasterTilesImpl) then,
  ) = __$$LayerConfig_RasterTilesImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String urlTemplate, String? attribution});
}

/// @nodoc
class __$$LayerConfig_RasterTilesImplCopyWithImpl<$Res>
    extends _$LayerConfigCopyWithImpl<$Res, _$LayerConfig_RasterTilesImpl>
    implements _$$LayerConfig_RasterTilesImplCopyWith<$Res> {
  __$$LayerConfig_RasterTilesImplCopyWithImpl(
    _$LayerConfig_RasterTilesImpl _value,
    $Res Function(_$LayerConfig_RasterTilesImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of LayerConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? urlTemplate = null, Object? attribution = freezed}) {
    return _then(
      _$LayerConfig_RasterTilesImpl(
        urlTemplate:
            null == urlTemplate
                ? _value.urlTemplate
                : urlTemplate // ignore: cast_nullable_to_non_nullable
                    as String,
        attribution:
            freezed == attribution
                ? _value.attribution
                : attribution // ignore: cast_nullable_to_non_nullable
                    as String?,
      ),
    );
  }
}

/// @nodoc

class _$LayerConfig_RasterTilesImpl extends LayerConfig_RasterTiles {
  const _$LayerConfig_RasterTilesImpl({
    required this.urlTemplate,
    this.attribution,
  }) : super._();

  @override
  final String urlTemplate;
  @override
  final String? attribution;

  @override
  String toString() {
    return 'LayerConfig.rasterTiles(urlTemplate: $urlTemplate, attribution: $attribution)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$LayerConfig_RasterTilesImpl &&
            (identical(other.urlTemplate, urlTemplate) ||
                other.urlTemplate == urlTemplate) &&
            (identical(other.attribution, attribution) ||
                other.attribution == attribution));
  }

  @override
  int get hashCode => Object.hash(runtimeType, urlTemplate, attribution);

  /// Create a copy of LayerConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$LayerConfig_RasterTilesImplCopyWith<_$LayerConfig_RasterTilesImpl>
  get copyWith => __$$LayerConfig_RasterTilesImplCopyWithImpl<
    _$LayerConfig_RasterTilesImpl
  >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() osm,
    required TResult Function(String urlTemplate, String? attribution)
    rasterTiles,
    required TResult Function(
      String urlTemplate,
      String styleJson,
      String? attribution,
    )
    vectorTiles,
  }) {
    return rasterTiles(urlTemplate, attribution);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? osm,
    TResult? Function(String urlTemplate, String? attribution)? rasterTiles,
    TResult? Function(
      String urlTemplate,
      String styleJson,
      String? attribution,
    )?
    vectorTiles,
  }) {
    return rasterTiles?.call(urlTemplate, attribution);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? osm,
    TResult Function(String urlTemplate, String? attribution)? rasterTiles,
    TResult Function(String urlTemplate, String styleJson, String? attribution)?
    vectorTiles,
    required TResult orElse(),
  }) {
    if (rasterTiles != null) {
      return rasterTiles(urlTemplate, attribution);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(LayerConfig_Osm value) osm,
    required TResult Function(LayerConfig_RasterTiles value) rasterTiles,
    required TResult Function(LayerConfig_VectorTiles value) vectorTiles,
  }) {
    return rasterTiles(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LayerConfig_Osm value)? osm,
    TResult? Function(LayerConfig_RasterTiles value)? rasterTiles,
    TResult? Function(LayerConfig_VectorTiles value)? vectorTiles,
  }) {
    return rasterTiles?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LayerConfig_Osm value)? osm,
    TResult Function(LayerConfig_RasterTiles value)? rasterTiles,
    TResult Function(LayerConfig_VectorTiles value)? vectorTiles,
    required TResult orElse(),
  }) {
    if (rasterTiles != null) {
      return rasterTiles(this);
    }
    return orElse();
  }
}

abstract class LayerConfig_RasterTiles extends LayerConfig {
  const factory LayerConfig_RasterTiles({
    required final String urlTemplate,
    final String? attribution,
  }) = _$LayerConfig_RasterTilesImpl;
  const LayerConfig_RasterTiles._() : super._();

  String get urlTemplate;
  String? get attribution;

  /// Create a copy of LayerConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$LayerConfig_RasterTilesImplCopyWith<_$LayerConfig_RasterTilesImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$LayerConfig_VectorTilesImplCopyWith<$Res> {
  factory _$$LayerConfig_VectorTilesImplCopyWith(
    _$LayerConfig_VectorTilesImpl value,
    $Res Function(_$LayerConfig_VectorTilesImpl) then,
  ) = __$$LayerConfig_VectorTilesImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String urlTemplate, String styleJson, String? attribution});
}

/// @nodoc
class __$$LayerConfig_VectorTilesImplCopyWithImpl<$Res>
    extends _$LayerConfigCopyWithImpl<$Res, _$LayerConfig_VectorTilesImpl>
    implements _$$LayerConfig_VectorTilesImplCopyWith<$Res> {
  __$$LayerConfig_VectorTilesImplCopyWithImpl(
    _$LayerConfig_VectorTilesImpl _value,
    $Res Function(_$LayerConfig_VectorTilesImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of LayerConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? urlTemplate = null,
    Object? styleJson = null,
    Object? attribution = freezed,
  }) {
    return _then(
      _$LayerConfig_VectorTilesImpl(
        urlTemplate:
            null == urlTemplate
                ? _value.urlTemplate
                : urlTemplate // ignore: cast_nullable_to_non_nullable
                    as String,
        styleJson:
            null == styleJson
                ? _value.styleJson
                : styleJson // ignore: cast_nullable_to_non_nullable
                    as String,
        attribution:
            freezed == attribution
                ? _value.attribution
                : attribution // ignore: cast_nullable_to_non_nullable
                    as String?,
      ),
    );
  }
}

/// @nodoc

class _$LayerConfig_VectorTilesImpl extends LayerConfig_VectorTiles {
  const _$LayerConfig_VectorTilesImpl({
    required this.urlTemplate,
    required this.styleJson,
    this.attribution,
  }) : super._();

  @override
  final String urlTemplate;
  @override
  final String styleJson;
  @override
  final String? attribution;

  @override
  String toString() {
    return 'LayerConfig.vectorTiles(urlTemplate: $urlTemplate, styleJson: $styleJson, attribution: $attribution)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$LayerConfig_VectorTilesImpl &&
            (identical(other.urlTemplate, urlTemplate) ||
                other.urlTemplate == urlTemplate) &&
            (identical(other.styleJson, styleJson) ||
                other.styleJson == styleJson) &&
            (identical(other.attribution, attribution) ||
                other.attribution == attribution));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, urlTemplate, styleJson, attribution);

  /// Create a copy of LayerConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$LayerConfig_VectorTilesImplCopyWith<_$LayerConfig_VectorTilesImpl>
  get copyWith => __$$LayerConfig_VectorTilesImplCopyWithImpl<
    _$LayerConfig_VectorTilesImpl
  >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() osm,
    required TResult Function(String urlTemplate, String? attribution)
    rasterTiles,
    required TResult Function(
      String urlTemplate,
      String styleJson,
      String? attribution,
    )
    vectorTiles,
  }) {
    return vectorTiles(urlTemplate, styleJson, attribution);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? osm,
    TResult? Function(String urlTemplate, String? attribution)? rasterTiles,
    TResult? Function(
      String urlTemplate,
      String styleJson,
      String? attribution,
    )?
    vectorTiles,
  }) {
    return vectorTiles?.call(urlTemplate, styleJson, attribution);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? osm,
    TResult Function(String urlTemplate, String? attribution)? rasterTiles,
    TResult Function(String urlTemplate, String styleJson, String? attribution)?
    vectorTiles,
    required TResult orElse(),
  }) {
    if (vectorTiles != null) {
      return vectorTiles(urlTemplate, styleJson, attribution);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(LayerConfig_Osm value) osm,
    required TResult Function(LayerConfig_RasterTiles value) rasterTiles,
    required TResult Function(LayerConfig_VectorTiles value) vectorTiles,
  }) {
    return vectorTiles(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LayerConfig_Osm value)? osm,
    TResult? Function(LayerConfig_RasterTiles value)? rasterTiles,
    TResult? Function(LayerConfig_VectorTiles value)? vectorTiles,
  }) {
    return vectorTiles?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LayerConfig_Osm value)? osm,
    TResult Function(LayerConfig_RasterTiles value)? rasterTiles,
    TResult Function(LayerConfig_VectorTiles value)? vectorTiles,
    required TResult orElse(),
  }) {
    if (vectorTiles != null) {
      return vectorTiles(this);
    }
    return orElse();
  }
}

abstract class LayerConfig_VectorTiles extends LayerConfig {
  const factory LayerConfig_VectorTiles({
    required final String urlTemplate,
    required final String styleJson,
    final String? attribution,
  }) = _$LayerConfig_VectorTilesImpl;
  const LayerConfig_VectorTiles._() : super._();

  String get urlTemplate;
  String get styleJson;
  String? get attribution;

  /// Create a copy of LayerConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$LayerConfig_VectorTilesImplCopyWith<_$LayerConfig_VectorTilesImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$UserEvent {
  Object get field0 => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UserEventCopyWith<$Res> {
  factory $UserEventCopyWith(UserEvent value, $Res Function(UserEvent) then) =
      _$UserEventCopyWithImpl<$Res, UserEvent>;
}

/// @nodoc
class _$UserEventCopyWithImpl<$Res, $Val extends UserEvent>
    implements $UserEventCopyWith<$Res> {
  _$UserEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$UserEvent_ButtonPressedImplCopyWith<$Res> {
  factory _$$UserEvent_ButtonPressedImplCopyWith(
    _$UserEvent_ButtonPressedImpl value,
    $Res Function(_$UserEvent_ButtonPressedImpl) then,
  ) = __$$UserEvent_ButtonPressedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({MouseButton field0, MouseEvent field1});
}

/// @nodoc
class __$$UserEvent_ButtonPressedImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_ButtonPressedImpl>
    implements _$$UserEvent_ButtonPressedImplCopyWith<$Res> {
  __$$UserEvent_ButtonPressedImplCopyWithImpl(
    _$UserEvent_ButtonPressedImpl _value,
    $Res Function(_$UserEvent_ButtonPressedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null, Object? field1 = null}) {
    return _then(
      _$UserEvent_ButtonPressedImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as MouseButton,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_ButtonPressedImpl extends UserEvent_ButtonPressed {
  const _$UserEvent_ButtonPressedImpl(this.field0, this.field1) : super._();

  @override
  final MouseButton field0;
  @override
  final MouseEvent field1;

  @override
  String toString() {
    return 'UserEvent.buttonPressed(field0: $field0, field1: $field1)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_ButtonPressedImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_ButtonPressedImplCopyWith<_$UserEvent_ButtonPressedImpl>
  get copyWith => __$$UserEvent_ButtonPressedImplCopyWithImpl<
    _$UserEvent_ButtonPressedImpl
  >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return buttonPressed(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return buttonPressed?.call(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (buttonPressed != null) {
      return buttonPressed(field0, field1);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return buttonPressed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return buttonPressed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (buttonPressed != null) {
      return buttonPressed(this);
    }
    return orElse();
  }
}

abstract class UserEvent_ButtonPressed extends UserEvent {
  const factory UserEvent_ButtonPressed(
    final MouseButton field0,
    final MouseEvent field1,
  ) = _$UserEvent_ButtonPressedImpl;
  const UserEvent_ButtonPressed._() : super._();

  @override
  MouseButton get field0;
  MouseEvent get field1;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_ButtonPressedImplCopyWith<_$UserEvent_ButtonPressedImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_ButtonReleasedImplCopyWith<$Res> {
  factory _$$UserEvent_ButtonReleasedImplCopyWith(
    _$UserEvent_ButtonReleasedImpl value,
    $Res Function(_$UserEvent_ButtonReleasedImpl) then,
  ) = __$$UserEvent_ButtonReleasedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({MouseButton field0, MouseEvent field1});
}

/// @nodoc
class __$$UserEvent_ButtonReleasedImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_ButtonReleasedImpl>
    implements _$$UserEvent_ButtonReleasedImplCopyWith<$Res> {
  __$$UserEvent_ButtonReleasedImplCopyWithImpl(
    _$UserEvent_ButtonReleasedImpl _value,
    $Res Function(_$UserEvent_ButtonReleasedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null, Object? field1 = null}) {
    return _then(
      _$UserEvent_ButtonReleasedImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as MouseButton,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_ButtonReleasedImpl extends UserEvent_ButtonReleased {
  const _$UserEvent_ButtonReleasedImpl(this.field0, this.field1) : super._();

  @override
  final MouseButton field0;
  @override
  final MouseEvent field1;

  @override
  String toString() {
    return 'UserEvent.buttonReleased(field0: $field0, field1: $field1)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_ButtonReleasedImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_ButtonReleasedImplCopyWith<_$UserEvent_ButtonReleasedImpl>
  get copyWith => __$$UserEvent_ButtonReleasedImplCopyWithImpl<
    _$UserEvent_ButtonReleasedImpl
  >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return buttonReleased(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return buttonReleased?.call(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (buttonReleased != null) {
      return buttonReleased(field0, field1);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return buttonReleased(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return buttonReleased?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (buttonReleased != null) {
      return buttonReleased(this);
    }
    return orElse();
  }
}

abstract class UserEvent_ButtonReleased extends UserEvent {
  const factory UserEvent_ButtonReleased(
    final MouseButton field0,
    final MouseEvent field1,
  ) = _$UserEvent_ButtonReleasedImpl;
  const UserEvent_ButtonReleased._() : super._();

  @override
  MouseButton get field0;
  MouseEvent get field1;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_ButtonReleasedImplCopyWith<_$UserEvent_ButtonReleasedImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_ClickImplCopyWith<$Res> {
  factory _$$UserEvent_ClickImplCopyWith(
    _$UserEvent_ClickImpl value,
    $Res Function(_$UserEvent_ClickImpl) then,
  ) = __$$UserEvent_ClickImplCopyWithImpl<$Res>;
  @useResult
  $Res call({MouseButton field0, MouseEvent field1});
}

/// @nodoc
class __$$UserEvent_ClickImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_ClickImpl>
    implements _$$UserEvent_ClickImplCopyWith<$Res> {
  __$$UserEvent_ClickImplCopyWithImpl(
    _$UserEvent_ClickImpl _value,
    $Res Function(_$UserEvent_ClickImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null, Object? field1 = null}) {
    return _then(
      _$UserEvent_ClickImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as MouseButton,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_ClickImpl extends UserEvent_Click {
  const _$UserEvent_ClickImpl(this.field0, this.field1) : super._();

  @override
  final MouseButton field0;
  @override
  final MouseEvent field1;

  @override
  String toString() {
    return 'UserEvent.click(field0: $field0, field1: $field1)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_ClickImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_ClickImplCopyWith<_$UserEvent_ClickImpl> get copyWith =>
      __$$UserEvent_ClickImplCopyWithImpl<_$UserEvent_ClickImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return click(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return click?.call(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (click != null) {
      return click(field0, field1);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return click(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return click?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (click != null) {
      return click(this);
    }
    return orElse();
  }
}

abstract class UserEvent_Click extends UserEvent {
  const factory UserEvent_Click(
    final MouseButton field0,
    final MouseEvent field1,
  ) = _$UserEvent_ClickImpl;
  const UserEvent_Click._() : super._();

  @override
  MouseButton get field0;
  MouseEvent get field1;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_ClickImplCopyWith<_$UserEvent_ClickImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_DoubleClickImplCopyWith<$Res> {
  factory _$$UserEvent_DoubleClickImplCopyWith(
    _$UserEvent_DoubleClickImpl value,
    $Res Function(_$UserEvent_DoubleClickImpl) then,
  ) = __$$UserEvent_DoubleClickImplCopyWithImpl<$Res>;
  @useResult
  $Res call({MouseButton field0, MouseEvent field1});
}

/// @nodoc
class __$$UserEvent_DoubleClickImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_DoubleClickImpl>
    implements _$$UserEvent_DoubleClickImplCopyWith<$Res> {
  __$$UserEvent_DoubleClickImplCopyWithImpl(
    _$UserEvent_DoubleClickImpl _value,
    $Res Function(_$UserEvent_DoubleClickImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null, Object? field1 = null}) {
    return _then(
      _$UserEvent_DoubleClickImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as MouseButton,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_DoubleClickImpl extends UserEvent_DoubleClick {
  const _$UserEvent_DoubleClickImpl(this.field0, this.field1) : super._();

  @override
  final MouseButton field0;
  @override
  final MouseEvent field1;

  @override
  String toString() {
    return 'UserEvent.doubleClick(field0: $field0, field1: $field1)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_DoubleClickImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_DoubleClickImplCopyWith<_$UserEvent_DoubleClickImpl>
  get copyWith =>
      __$$UserEvent_DoubleClickImplCopyWithImpl<_$UserEvent_DoubleClickImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return doubleClick(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return doubleClick?.call(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (doubleClick != null) {
      return doubleClick(field0, field1);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return doubleClick(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return doubleClick?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (doubleClick != null) {
      return doubleClick(this);
    }
    return orElse();
  }
}

abstract class UserEvent_DoubleClick extends UserEvent {
  const factory UserEvent_DoubleClick(
    final MouseButton field0,
    final MouseEvent field1,
  ) = _$UserEvent_DoubleClickImpl;
  const UserEvent_DoubleClick._() : super._();

  @override
  MouseButton get field0;
  MouseEvent get field1;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_DoubleClickImplCopyWith<_$UserEvent_DoubleClickImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_PointerMovedImplCopyWith<$Res> {
  factory _$$UserEvent_PointerMovedImplCopyWith(
    _$UserEvent_PointerMovedImpl value,
    $Res Function(_$UserEvent_PointerMovedImpl) then,
  ) = __$$UserEvent_PointerMovedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({MouseEvent field0});
}

/// @nodoc
class __$$UserEvent_PointerMovedImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_PointerMovedImpl>
    implements _$$UserEvent_PointerMovedImplCopyWith<$Res> {
  __$$UserEvent_PointerMovedImplCopyWithImpl(
    _$UserEvent_PointerMovedImpl _value,
    $Res Function(_$UserEvent_PointerMovedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null}) {
    return _then(
      _$UserEvent_PointerMovedImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_PointerMovedImpl extends UserEvent_PointerMoved {
  const _$UserEvent_PointerMovedImpl(this.field0) : super._();

  @override
  final MouseEvent field0;

  @override
  String toString() {
    return 'UserEvent.pointerMoved(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_PointerMovedImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_PointerMovedImplCopyWith<_$UserEvent_PointerMovedImpl>
  get copyWith =>
      __$$UserEvent_PointerMovedImplCopyWithImpl<_$UserEvent_PointerMovedImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return pointerMoved(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return pointerMoved?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (pointerMoved != null) {
      return pointerMoved(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return pointerMoved(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return pointerMoved?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (pointerMoved != null) {
      return pointerMoved(this);
    }
    return orElse();
  }
}

abstract class UserEvent_PointerMoved extends UserEvent {
  const factory UserEvent_PointerMoved(final MouseEvent field0) =
      _$UserEvent_PointerMovedImpl;
  const UserEvent_PointerMoved._() : super._();

  @override
  MouseEvent get field0;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_PointerMovedImplCopyWith<_$UserEvent_PointerMovedImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_DragStartedImplCopyWith<$Res> {
  factory _$$UserEvent_DragStartedImplCopyWith(
    _$UserEvent_DragStartedImpl value,
    $Res Function(_$UserEvent_DragStartedImpl) then,
  ) = __$$UserEvent_DragStartedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({MouseButton field0, MouseEvent field1});
}

/// @nodoc
class __$$UserEvent_DragStartedImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_DragStartedImpl>
    implements _$$UserEvent_DragStartedImplCopyWith<$Res> {
  __$$UserEvent_DragStartedImplCopyWithImpl(
    _$UserEvent_DragStartedImpl _value,
    $Res Function(_$UserEvent_DragStartedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null, Object? field1 = null}) {
    return _then(
      _$UserEvent_DragStartedImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as MouseButton,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_DragStartedImpl extends UserEvent_DragStarted {
  const _$UserEvent_DragStartedImpl(this.field0, this.field1) : super._();

  @override
  final MouseButton field0;
  @override
  final MouseEvent field1;

  @override
  String toString() {
    return 'UserEvent.dragStarted(field0: $field0, field1: $field1)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_DragStartedImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_DragStartedImplCopyWith<_$UserEvent_DragStartedImpl>
  get copyWith =>
      __$$UserEvent_DragStartedImplCopyWithImpl<_$UserEvent_DragStartedImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return dragStarted(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return dragStarted?.call(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (dragStarted != null) {
      return dragStarted(field0, field1);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return dragStarted(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return dragStarted?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (dragStarted != null) {
      return dragStarted(this);
    }
    return orElse();
  }
}

abstract class UserEvent_DragStarted extends UserEvent {
  const factory UserEvent_DragStarted(
    final MouseButton field0,
    final MouseEvent field1,
  ) = _$UserEvent_DragStartedImpl;
  const UserEvent_DragStarted._() : super._();

  @override
  MouseButton get field0;
  MouseEvent get field1;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_DragStartedImplCopyWith<_$UserEvent_DragStartedImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_DragImplCopyWith<$Res> {
  factory _$$UserEvent_DragImplCopyWith(
    _$UserEvent_DragImpl value,
    $Res Function(_$UserEvent_DragImpl) then,
  ) = __$$UserEvent_DragImplCopyWithImpl<$Res>;
  @useResult
  $Res call({MouseButton field0, Vector2 field1, MouseEvent field2});
}

/// @nodoc
class __$$UserEvent_DragImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_DragImpl>
    implements _$$UserEvent_DragImplCopyWith<$Res> {
  __$$UserEvent_DragImplCopyWithImpl(
    _$UserEvent_DragImpl _value,
    $Res Function(_$UserEvent_DragImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
    Object? field1 = null,
    Object? field2 = null,
  }) {
    return _then(
      _$UserEvent_DragImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as MouseButton,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as Vector2,
        null == field2
            ? _value.field2
            : field2 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_DragImpl extends UserEvent_Drag {
  const _$UserEvent_DragImpl(this.field0, this.field1, this.field2) : super._();

  @override
  final MouseButton field0;
  @override
  final Vector2 field1;
  @override
  final MouseEvent field2;

  @override
  String toString() {
    return 'UserEvent.drag(field0: $field0, field1: $field1, field2: $field2)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_DragImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1) &&
            (identical(other.field2, field2) || other.field2 == field2));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1, field2);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_DragImplCopyWith<_$UserEvent_DragImpl> get copyWith =>
      __$$UserEvent_DragImplCopyWithImpl<_$UserEvent_DragImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return drag(field0, field1, field2);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return drag?.call(field0, field1, field2);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (drag != null) {
      return drag(field0, field1, field2);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return drag(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return drag?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (drag != null) {
      return drag(this);
    }
    return orElse();
  }
}

abstract class UserEvent_Drag extends UserEvent {
  const factory UserEvent_Drag(
    final MouseButton field0,
    final Vector2 field1,
    final MouseEvent field2,
  ) = _$UserEvent_DragImpl;
  const UserEvent_Drag._() : super._();

  @override
  MouseButton get field0;
  Vector2 get field1;
  MouseEvent get field2;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_DragImplCopyWith<_$UserEvent_DragImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_DragEndedImplCopyWith<$Res> {
  factory _$$UserEvent_DragEndedImplCopyWith(
    _$UserEvent_DragEndedImpl value,
    $Res Function(_$UserEvent_DragEndedImpl) then,
  ) = __$$UserEvent_DragEndedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({MouseButton field0, MouseEvent field1});
}

/// @nodoc
class __$$UserEvent_DragEndedImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_DragEndedImpl>
    implements _$$UserEvent_DragEndedImplCopyWith<$Res> {
  __$$UserEvent_DragEndedImplCopyWithImpl(
    _$UserEvent_DragEndedImpl _value,
    $Res Function(_$UserEvent_DragEndedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null, Object? field1 = null}) {
    return _then(
      _$UserEvent_DragEndedImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as MouseButton,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_DragEndedImpl extends UserEvent_DragEnded {
  const _$UserEvent_DragEndedImpl(this.field0, this.field1) : super._();

  @override
  final MouseButton field0;
  @override
  final MouseEvent field1;

  @override
  String toString() {
    return 'UserEvent.dragEnded(field0: $field0, field1: $field1)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_DragEndedImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_DragEndedImplCopyWith<_$UserEvent_DragEndedImpl> get copyWith =>
      __$$UserEvent_DragEndedImplCopyWithImpl<_$UserEvent_DragEndedImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return dragEnded(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return dragEnded?.call(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (dragEnded != null) {
      return dragEnded(field0, field1);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return dragEnded(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return dragEnded?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (dragEnded != null) {
      return dragEnded(this);
    }
    return orElse();
  }
}

abstract class UserEvent_DragEnded extends UserEvent {
  const factory UserEvent_DragEnded(
    final MouseButton field0,
    final MouseEvent field1,
  ) = _$UserEvent_DragEndedImpl;
  const UserEvent_DragEnded._() : super._();

  @override
  MouseButton get field0;
  MouseEvent get field1;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_DragEndedImplCopyWith<_$UserEvent_DragEndedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_ScrollImplCopyWith<$Res> {
  factory _$$UserEvent_ScrollImplCopyWith(
    _$UserEvent_ScrollImpl value,
    $Res Function(_$UserEvent_ScrollImpl) then,
  ) = __$$UserEvent_ScrollImplCopyWithImpl<$Res>;
  @useResult
  $Res call({double field0, MouseEvent field1});
}

/// @nodoc
class __$$UserEvent_ScrollImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_ScrollImpl>
    implements _$$UserEvent_ScrollImplCopyWith<$Res> {
  __$$UserEvent_ScrollImplCopyWithImpl(
    _$UserEvent_ScrollImpl _value,
    $Res Function(_$UserEvent_ScrollImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null, Object? field1 = null}) {
    return _then(
      _$UserEvent_ScrollImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as double,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as MouseEvent,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_ScrollImpl extends UserEvent_Scroll {
  const _$UserEvent_ScrollImpl(this.field0, this.field1) : super._();

  @override
  final double field0;
  @override
  final MouseEvent field1;

  @override
  String toString() {
    return 'UserEvent.scroll(field0: $field0, field1: $field1)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_ScrollImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_ScrollImplCopyWith<_$UserEvent_ScrollImpl> get copyWith =>
      __$$UserEvent_ScrollImplCopyWithImpl<_$UserEvent_ScrollImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return scroll(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return scroll?.call(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (scroll != null) {
      return scroll(field0, field1);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return scroll(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return scroll?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (scroll != null) {
      return scroll(this);
    }
    return orElse();
  }
}

abstract class UserEvent_Scroll extends UserEvent {
  const factory UserEvent_Scroll(final double field0, final MouseEvent field1) =
      _$UserEvent_ScrollImpl;
  const UserEvent_Scroll._() : super._();

  @override
  double get field0;
  MouseEvent get field1;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_ScrollImplCopyWith<_$UserEvent_ScrollImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UserEvent_ZoomImplCopyWith<$Res> {
  factory _$$UserEvent_ZoomImplCopyWith(
    _$UserEvent_ZoomImpl value,
    $Res Function(_$UserEvent_ZoomImpl) then,
  ) = __$$UserEvent_ZoomImplCopyWithImpl<$Res>;
  @useResult
  $Res call({double field0, Point2 field1});
}

/// @nodoc
class __$$UserEvent_ZoomImplCopyWithImpl<$Res>
    extends _$UserEventCopyWithImpl<$Res, _$UserEvent_ZoomImpl>
    implements _$$UserEvent_ZoomImplCopyWith<$Res> {
  __$$UserEvent_ZoomImplCopyWithImpl(
    _$UserEvent_ZoomImpl _value,
    $Res Function(_$UserEvent_ZoomImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null, Object? field1 = null}) {
    return _then(
      _$UserEvent_ZoomImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                as double,
        null == field1
            ? _value.field1
            : field1 // ignore: cast_nullable_to_non_nullable
                as Point2,
      ),
    );
  }
}

/// @nodoc

class _$UserEvent_ZoomImpl extends UserEvent_Zoom {
  const _$UserEvent_ZoomImpl(this.field0, this.field1) : super._();

  @override
  final double field0;
  @override
  final Point2 field1;

  @override
  String toString() {
    return 'UserEvent.zoom(field0: $field0, field1: $field1)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserEvent_ZoomImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            (identical(other.field1, field1) || other.field1 == field1));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0, field1);

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserEvent_ZoomImplCopyWith<_$UserEvent_ZoomImpl> get copyWith =>
      __$$UserEvent_ZoomImplCopyWithImpl<_$UserEvent_ZoomImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonPressed,
    required TResult Function(MouseButton field0, MouseEvent field1)
    buttonReleased,
    required TResult Function(MouseButton field0, MouseEvent field1) click,
    required TResult Function(MouseButton field0, MouseEvent field1)
    doubleClick,
    required TResult Function(MouseEvent field0) pointerMoved,
    required TResult Function(MouseButton field0, MouseEvent field1)
    dragStarted,
    required TResult Function(
      MouseButton field0,
      Vector2 field1,
      MouseEvent field2,
    )
    drag,
    required TResult Function(MouseButton field0, MouseEvent field1) dragEnded,
    required TResult Function(double field0, MouseEvent field1) scroll,
    required TResult Function(double field0, Point2 field1) zoom,
  }) {
    return zoom(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult? Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult? Function(MouseButton field0, MouseEvent field1)? click,
    TResult? Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult? Function(MouseEvent field0)? pointerMoved,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult? Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult? Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult? Function(double field0, MouseEvent field1)? scroll,
    TResult? Function(double field0, Point2 field1)? zoom,
  }) {
    return zoom?.call(field0, field1);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(MouseButton field0, MouseEvent field1)? buttonPressed,
    TResult Function(MouseButton field0, MouseEvent field1)? buttonReleased,
    TResult Function(MouseButton field0, MouseEvent field1)? click,
    TResult Function(MouseButton field0, MouseEvent field1)? doubleClick,
    TResult Function(MouseEvent field0)? pointerMoved,
    TResult Function(MouseButton field0, MouseEvent field1)? dragStarted,
    TResult Function(MouseButton field0, Vector2 field1, MouseEvent field2)?
    drag,
    TResult Function(MouseButton field0, MouseEvent field1)? dragEnded,
    TResult Function(double field0, MouseEvent field1)? scroll,
    TResult Function(double field0, Point2 field1)? zoom,
    required TResult orElse(),
  }) {
    if (zoom != null) {
      return zoom(field0, field1);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UserEvent_ButtonPressed value) buttonPressed,
    required TResult Function(UserEvent_ButtonReleased value) buttonReleased,
    required TResult Function(UserEvent_Click value) click,
    required TResult Function(UserEvent_DoubleClick value) doubleClick,
    required TResult Function(UserEvent_PointerMoved value) pointerMoved,
    required TResult Function(UserEvent_DragStarted value) dragStarted,
    required TResult Function(UserEvent_Drag value) drag,
    required TResult Function(UserEvent_DragEnded value) dragEnded,
    required TResult Function(UserEvent_Scroll value) scroll,
    required TResult Function(UserEvent_Zoom value) zoom,
  }) {
    return zoom(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult? Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult? Function(UserEvent_Click value)? click,
    TResult? Function(UserEvent_DoubleClick value)? doubleClick,
    TResult? Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult? Function(UserEvent_DragStarted value)? dragStarted,
    TResult? Function(UserEvent_Drag value)? drag,
    TResult? Function(UserEvent_DragEnded value)? dragEnded,
    TResult? Function(UserEvent_Scroll value)? scroll,
    TResult? Function(UserEvent_Zoom value)? zoom,
  }) {
    return zoom?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UserEvent_ButtonPressed value)? buttonPressed,
    TResult Function(UserEvent_ButtonReleased value)? buttonReleased,
    TResult Function(UserEvent_Click value)? click,
    TResult Function(UserEvent_DoubleClick value)? doubleClick,
    TResult Function(UserEvent_PointerMoved value)? pointerMoved,
    TResult Function(UserEvent_DragStarted value)? dragStarted,
    TResult Function(UserEvent_Drag value)? drag,
    TResult Function(UserEvent_DragEnded value)? dragEnded,
    TResult Function(UserEvent_Scroll value)? scroll,
    TResult Function(UserEvent_Zoom value)? zoom,
    required TResult orElse(),
  }) {
    if (zoom != null) {
      return zoom(this);
    }
    return orElse();
  }
}

abstract class UserEvent_Zoom extends UserEvent {
  const factory UserEvent_Zoom(final double field0, final Point2 field1) =
      _$UserEvent_ZoomImpl;
  const UserEvent_Zoom._() : super._();

  @override
  double get field0;
  Point2 get field1;

  /// Create a copy of UserEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserEvent_ZoomImplCopyWith<_$UserEvent_ZoomImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
