// Generated by the protocol buffer compiler. DO NOT EDIT!
// source: transmit.proto

// Generated files should ignore deprecation warnings
@file:Suppress("DEPRECATION")
package transmit;

@kotlin.jvm.JvmName("-initializescheduleTransmissionResponse")
public inline fun scheduleTransmissionResponse(block: transmit.ScheduleTransmissionResponseKt.Dsl.() -> kotlin.Unit): transmit.TransmitOuterClass.ScheduleTransmissionResponse =
  transmit.ScheduleTransmissionResponseKt.Dsl._create(transmit.TransmitOuterClass.ScheduleTransmissionResponse.newBuilder()).apply { block() }._build()
/**
 * Protobuf type `transmit.ScheduleTransmissionResponse`
 */
public object ScheduleTransmissionResponseKt {
  @kotlin.OptIn(com.google.protobuf.kotlin.OnlyForUseByGeneratedProtoCode::class)
  @com.google.protobuf.kotlin.ProtoDslMarker
  public class Dsl private constructor(
    private val _builder: transmit.TransmitOuterClass.ScheduleTransmissionResponse.Builder
  ) {
    public companion object {
      @kotlin.jvm.JvmSynthetic
      @kotlin.PublishedApi
      internal fun _create(builder: transmit.TransmitOuterClass.ScheduleTransmissionResponse.Builder): Dsl = Dsl(builder)
    }

    @kotlin.jvm.JvmSynthetic
    @kotlin.PublishedApi
    internal fun _build(): transmit.TransmitOuterClass.ScheduleTransmissionResponse = _builder.build()

    /**
     * `string transmission_id = 1;`
     */
    public var transmissionId: kotlin.String
      @JvmName("getTransmissionId")
      get() = _builder.getTransmissionId()
      @JvmName("setTransmissionId")
      set(value) {
        _builder.setTransmissionId(value)
      }
    /**
     * `string transmission_id = 1;`
     */
    public fun clearTransmissionId() {
      _builder.clearTransmissionId()
    }
  }
}
@kotlin.jvm.JvmSynthetic
public inline fun transmit.TransmitOuterClass.ScheduleTransmissionResponse.copy(block: `transmit`.ScheduleTransmissionResponseKt.Dsl.() -> kotlin.Unit): transmit.TransmitOuterClass.ScheduleTransmissionResponse =
  `transmit`.ScheduleTransmissionResponseKt.Dsl._create(this.toBuilder()).apply { block() }._build()

