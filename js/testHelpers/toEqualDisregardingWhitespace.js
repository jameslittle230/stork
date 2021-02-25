import { matcherHint, printReceived, printExpected } from "jest-matcher-utils";
import diff from "jest-diff";

const toEqualDisregardingWhitespace = (received, expected) => {
  const compressWhitespace = str => str.replace(/\s+/g, ``);

  const [received_compressed, expected_compressed] = [received, expected].map(
    compressWhitespace
  );

  const pass = received_compressed == expected_compressed;

  const message = pass
    ? () =>
        `${matcherHint(`.not.${name}`)}\n\n` +
        `Uncompressed expected value:\n` +
        `  ${printExpected(expected)}\n` +
        `Expected value with compressed whitespace to not equal:\n` +
        `  ${printExpected(expected_compressed)}\n` +
        `Uncompressed received value:\n` +
        `  ${printReceived(received)}\n` +
        `Received value with compressed whitespace:\n` +
        `  ${printReceived(received_compressed)}`
    : () => {
        const diffString = diff(expected_compressed, received_compressed, {
          expand: this.expand
        });
        return (
          `${matcherHint(`.${name}`)}\n\n` +
          `Uncompressed expected value:\n` +
          `  ${printExpected(expected)}\n` +
          `Expected value with compressed whitespace to equal:\n` +
          `  ${printExpected(expected_compressed)}\n` +
          `Uncompressed received value:\n` +
          `  ${printReceived(received)}\n` +
          `Received value with compressed whitespace:\n` +
          `  ${printReceived(received_compressed)}${
            diffString ? `\n\nDifference:\n\n${diffString}` : ``
          }`
        );
      };
  return {
    actual: received,
    expected,
    message,
    name,
    pass
  };
};

export default toEqualDisregardingWhitespace;
