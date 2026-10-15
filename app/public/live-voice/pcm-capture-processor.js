/**
 * Bounded-memory streaming PCM16 worklet for live voice microphone uplink.
 * The public worklet URL, name, 16kHz sample rate, 100ms frame contract and
 * binary ArrayBuffer message shape are preserved. No per-quantum Float32Array
 * concatenation/copy is needed; output buffers transfer ownership at frame
 * boundaries (about ten allocations per second for 100ms uplink frames).
 *
 * For downsampling, use a fractional-width box average to retain the
 * previous low-pass behavior without an unread carry buffer.
 * For upsampling, interpolate consecutive input samples across quanta with an
 * absolute fractional output clock to avoid drift.
 */
/* global sampleRate, registerProcessor, AudioWorkletProcessor */
class PcmCaptureProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    const opts = (options && options.processorOptions) || {};
    this.targetRate = opts.targetSampleRate || 16000;
    this.frameSamples = opts.frameSamples || 1600;
    this.ratio = sampleRate / this.targetRate;
    this.frame = new Int16Array(this.frameSamples);
    this.frameFill = 0;
    this.accumulatedInput = 0;
    this.accumulatedValue = 0;
    this.hasPrevious = false;
    this.previousInput = 0;
    this.inputSampleIndex = 0;
    this.nextOutputInputPosition = 0;
  }

  pushSample(value) {
    const clamped = value > 1 ? 1 : value < -1 ? -1 : value;
    this.frame[this.frameFill] = clamped < 0 ? clamped * 0x8000 : clamped * 0x7fff;
    this.frameFill++;
    if (this.frameFill === this.frameSamples) {
      const completed = this.frame.buffer;
      this.frame = new Int16Array(this.frameSamples);
      this.frameFill = 0;
      this.port.postMessage(completed, [completed]);
    }
  }

  process(inputs) {
    const channel = inputs[0] && inputs[0][0];
    if (!channel || channel.length === 0) return true;

    if (this.ratio >= 1) {
      for (let i = 0; i < channel.length; i++) {
        const value = channel[i];
        let remaining = 1;
        while (remaining > 1e-12) {
          const width = Math.min(remaining, this.ratio - this.accumulatedInput);
          this.accumulatedValue += value * width;
          this.accumulatedInput += width;
          remaining -= width;
          if (this.accumulatedInput >= this.ratio - 1e-10) {
            this.pushSample(this.accumulatedValue / this.ratio);
            this.accumulatedInput = 0;
            this.accumulatedValue = 0;
          }
        }
      }
    } else {
      for (let i = 0; i < channel.length; i++) {
        const current = channel[i];
        if (!this.hasPrevious) {
          this.hasPrevious = true;
          this.previousInput = current;
          this.pushSample(current);
          this.nextOutputInputPosition = this.ratio;
          continue;
        }
        const right = ++this.inputSampleIndex;
        const left = right - 1;
        while (this.nextOutputInputPosition <= right + 1e-10) {
          const fraction = Math.min(1, Math.max(0, this.nextOutputInputPosition - left));
          this.pushSample(this.previousInput + (current - this.previousInput) * fraction);
          this.nextOutputInputPosition += this.ratio;
        }
        this.previousInput = current;
      }
    }
    return true;
  }
}

registerProcessor('openhuman-pcm-capture', PcmCaptureProcessor);
