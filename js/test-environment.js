// This helper defines the TextEncoder field for jsdom.
// https://stackoverflow.com/a/57713960/3841018
const Environment = require("jest-environment-jsdom");
module.exports = class CustomTestEnvironment extends Environment {
  async setup() {
    await super.setup();
    if (typeof this.global.TextEncoder === "undefined") {
      const { TextEncoder, TextDecoder } = require("util");
      this.global.TextEncoder = TextEncoder;
      this.global.TextDecoder = TextDecoder;
    }
  }
};
