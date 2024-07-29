const DEFAULT_GAIN = 0.08;

export default class Buzzer {
  ctx: AudioContext;
  osc: OscillatorNode | undefined;
  gainNode: GainNode;

  constructor() {
    this.ctx = new AudioContext();
    this.gainNode = this.ctx.createGain();
    this.gainNode.connect(this.ctx.destination);
    this.gainNode.gain.value = DEFAULT_GAIN;
  }

  play() {
    if (!this.osc) {
      this.osc = this.ctx.createOscillator();
      this.osc.type = "square";
      this.osc.frequency.value = 261.63;
      this.osc.connect(this.gainNode);
      this.osc.start();
    }
  }

  stop() {
    if (this.osc) {
      this.osc.stop();
      this.osc.disconnect();
      this.osc = undefined;
    }
  }

  mute() {
    this.gainNode.gain.value = 0;
  }

  unmute() {
    this.gainNode.gain.value = DEFAULT_GAIN;
  }
}
