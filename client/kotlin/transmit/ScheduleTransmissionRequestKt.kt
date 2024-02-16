// Generated by the protocol buffer compiler. DO NOT EDIT!
// source: transmit.proto

// Generated files should ignore deprecation warnings
@file:Suppress("DEPRECATION")
package transmit;

@kotlin.jvm.JvmName("-initializescheduleTransmissionRequest")
public inline fun scheduleTransmissionRequest(block: transmit.ScheduleTransmissionRequestKt.Dsl.() -> kotlin.Unit): transmit.TransmitOuterClass.ScheduleTransmissionRequest =
  transmit.ScheduleTransmissionRequestKt.Dsl._create(transmit.TransmitOuterClass.ScheduleTransmissionRequest.newBuilder()).apply { block() }._build()
/**
 * Protobuf type `transmit.ScheduleTransmissionRequest`
 */
public object ScheduleTransmissionRequestKt {
  @kotlin.OptIn(com.google.protobuf.kotlin.OnlyForUseByGeneratedProtoCode::class)
  @com.google.protobuf.kotlin.ProtoDslMarker
  public class Dsl private constructor(
    private val _builder: transmit.TransmitOuterClass.ScheduleTransmissionRequest.Builder
  ) {
    public companion object {
      @kotlin.jvm.JvmSynthetic
      @kotlin.PublishedApi
      internal fun _create(builder: transmit.TransmitOuterClass.ScheduleTransmissionRequest.Builder): Dsl = Dsl(builder)
    }

    @kotlin.jvm.JvmSynthetic
    @kotlin.PublishedApi
    internal fun _build(): transmit.TransmitOuterClass.ScheduleTransmissionRequest = _builder.build()

    /**
     * `.transmit.Delayed delayed = 1;`
     */
    public var delayed: transmit.TransmitOuterClass.Delayed
      @JvmName("getDelayed")
      get() = _builder.getDelayed()
      @JvmName("setDelayed")
      set(value) {
        _builder.setDelayed(value)
      }
    /**
     * `.transmit.Delayed delayed = 1;`
     */
    public fun clearDelayed() {
      _builder.clearDelayed()
    }
    /**
     * `.transmit.Delayed delayed = 1;`
     * @return Whether the delayed field is set.
     */
    public fun hasDelayed(): kotlin.Boolean {
      return _builder.hasDelayed()
    }

    /**
     * `.transmit.Interval interval = 2;`
     */
    public var interval: transmit.TransmitOuterClass.Interval
      @JvmName("getInterval")
      get() = _builder.getInterval()
      @JvmName("setInterval")
      set(value) {
        _builder.setInterval(value)
      }
    /**
     * `.transmit.Interval interval = 2;`
     */
    public fun clearInterval() {
      _builder.clearInterval()
    }
    /**
     * `.transmit.Interval interval = 2;`
     * @return Whether the interval field is set.
     */
    public fun hasInterval(): kotlin.Boolean {
      return _builder.hasInterval()
    }

    /**
     * `.transmit.Cron cron = 3;`
     */
    public var cron: transmit.TransmitOuterClass.Cron
      @JvmName("getCron")
      get() = _builder.getCron()
      @JvmName("setCron")
      set(value) {
        _builder.setCron(value)
      }
    /**
     * `.transmit.Cron cron = 3;`
     */
    public fun clearCron() {
      _builder.clearCron()
    }
    /**
     * `.transmit.Cron cron = 3;`
     * @return Whether the cron field is set.
     */
    public fun hasCron(): kotlin.Boolean {
      return _builder.hasCron()
    }

    /**
     * `.transmit.NatsEvent nats_event = 4;`
     */
    public var natsEvent: transmit.TransmitOuterClass.NatsEvent
      @JvmName("getNatsEvent")
      get() = _builder.getNatsEvent()
      @JvmName("setNatsEvent")
      set(value) {
        _builder.setNatsEvent(value)
      }
    /**
     * `.transmit.NatsEvent nats_event = 4;`
     */
    public fun clearNatsEvent() {
      _builder.clearNatsEvent()
    }
    /**
     * `.transmit.NatsEvent nats_event = 4;`
     * @return Whether the natsEvent field is set.
     */
    public fun hasNatsEvent(): kotlin.Boolean {
      return _builder.hasNatsEvent()
    }
    public val scheduleCase: transmit.TransmitOuterClass.ScheduleTransmissionRequest.ScheduleCase
      @JvmName("getScheduleCase")
      get() = _builder.getScheduleCase()

    public fun clearSchedule() {
      _builder.clearSchedule()
    }
    public val messageCase: transmit.TransmitOuterClass.ScheduleTransmissionRequest.MessageCase
      @JvmName("getMessageCase")
      get() = _builder.getMessageCase()

    public fun clearMessage() {
      _builder.clearMessage()
    }
  }
}
@kotlin.jvm.JvmSynthetic
public inline fun transmit.TransmitOuterClass.ScheduleTransmissionRequest.copy(block: `transmit`.ScheduleTransmissionRequestKt.Dsl.() -> kotlin.Unit): transmit.TransmitOuterClass.ScheduleTransmissionRequest =
  `transmit`.ScheduleTransmissionRequestKt.Dsl._create(this.toBuilder()).apply { block() }._build()

public val transmit.TransmitOuterClass.ScheduleTransmissionRequestOrBuilder.delayedOrNull: transmit.TransmitOuterClass.Delayed?
  get() = if (hasDelayed()) getDelayed() else null

public val transmit.TransmitOuterClass.ScheduleTransmissionRequestOrBuilder.intervalOrNull: transmit.TransmitOuterClass.Interval?
  get() = if (hasInterval()) getInterval() else null

public val transmit.TransmitOuterClass.ScheduleTransmissionRequestOrBuilder.cronOrNull: transmit.TransmitOuterClass.Cron?
  get() = if (hasCron()) getCron() else null

public val transmit.TransmitOuterClass.ScheduleTransmissionRequestOrBuilder.natsEventOrNull: transmit.TransmitOuterClass.NatsEvent?
  get() = if (hasNatsEvent()) getNatsEvent() else null

