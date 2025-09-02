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
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? osm,
    TResult? Function(String urlTemplate, String? attribution)? rasterTiles,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? osm,
    TResult Function(String urlTemplate, String? attribution)? rasterTiles,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(LayerConfig_Osm value) osm,
    required TResult Function(LayerConfig_RasterTiles value) rasterTiles,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LayerConfig_Osm value)? osm,
    TResult? Function(LayerConfig_RasterTiles value)? rasterTiles,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LayerConfig_Osm value)? osm,
    TResult Function(LayerConfig_RasterTiles value)? rasterTiles,
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
  }) {
    return osm();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? osm,
    TResult? Function(String urlTemplate, String? attribution)? rasterTiles,
  }) {
    return osm?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? osm,
    TResult Function(String urlTemplate, String? attribution)? rasterTiles,
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
  }) {
    return osm(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LayerConfig_Osm value)? osm,
    TResult? Function(LayerConfig_RasterTiles value)? rasterTiles,
  }) {
    return osm?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LayerConfig_Osm value)? osm,
    TResult Function(LayerConfig_RasterTiles value)? rasterTiles,
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
  }) {
    return rasterTiles(urlTemplate, attribution);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? osm,
    TResult? Function(String urlTemplate, String? attribution)? rasterTiles,
  }) {
    return rasterTiles?.call(urlTemplate, attribution);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? osm,
    TResult Function(String urlTemplate, String? attribution)? rasterTiles,
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
  }) {
    return rasterTiles(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LayerConfig_Osm value)? osm,
    TResult? Function(LayerConfig_RasterTiles value)? rasterTiles,
  }) {
    return rasterTiles?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LayerConfig_Osm value)? osm,
    TResult Function(LayerConfig_RasterTiles value)? rasterTiles,
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
