// Generated by the protocol buffer compiler. DO NOT EDIT!
// source: transmit.proto

// Generated files should ignore deprecation warnings
@file:Suppress("DEPRECATION")
package transmit;

@kotlin.jvm.JvmName("-initializehealthCheckRequest")
public inline fun healthCheckRequest(block: transmit.HealthCheckRequestKt.Dsl.() -> kotlin.Unit): transmit.TransmitOuterClass.HealthCheckRequest =
  transmit.HealthCheckRequestKt.Dsl._create(transmit.TransmitOuterClass.HealthCheckRequest.newBuilder()).apply { block() }._build()
/**
 * Protobuf type `transmit.HealthCheckRequest`
 */
public object HealthCheckRequestKt {
  @kotlin.OptIn(com.google.protobuf.kotlin.OnlyForUseByGeneratedProtoCode::class)
  @com.google.protobuf.kotlin.ProtoDslMarker
  public class Dsl private constructor(
    private val _builder: transmit.TransmitOuterClass.HealthCheckRequest.Builder
  ) {
    public companion object {
      @kotlin.jvm.JvmSynthetic
      @kotlin.PublishedApi
      internal fun _create(builder: transmit.TransmitOuterClass.HealthCheckRequest.Builder): Dsl = Dsl(builder)
    }

    @kotlin.jvm.JvmSynthetic
    @kotlin.PublishedApi
    internal fun _build(): transmit.TransmitOuterClass.HealthCheckRequest = _builder.build()

    /**
     * `string service = 1;`
     */
    public var service: kotlin.String
      @JvmName("getService")
      get() = _builder.getService()
      @JvmName("setService")
      set(value) {
        _builder.setService(value)
      }
    /**
     * `string service = 1;`
     */
    public fun clearService() {
      _builder.clearService()
    }
  }
}
@kotlin.jvm.JvmSynthetic
public inline fun transmit.TransmitOuterClass.HealthCheckRequest.copy(block: `transmit`.HealthCheckRequestKt.Dsl.() -> kotlin.Unit): transmit.TransmitOuterClass.HealthCheckRequest =
  `transmit`.HealthCheckRequestKt.Dsl._create(this.toBuilder()).apply { block() }._build()

