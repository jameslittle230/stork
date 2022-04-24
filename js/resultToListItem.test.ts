import { resultToListItem } from "./resultToListItem";
import { Result } from "./searchData";
import toEqualDisregardingWhitespace from "./testHelpers/toEqualDisregardingWhitespace";

expect.extend({
  toEqualDisregardingWhitespace: toEqualDisregardingWhitespace
});

test("resultToListItem happy path", () => {
  const result: Result = {
    entry: {
      fields: {},
      title: "Result Title",
      url: "https://jameslittle.me"
    },
    excerpts: [
      { fields: {}, score: 12, text: "This is the text of the excerpt." }
    ],
    score: 12,
    title_highlight_ranges: []
  };

  const node = resultToListItem(result, { selected: false, showScores: false });
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  expect((node as Element).innerHTML).toEqualDisregardingWhitespace(
    `<a href="https://jameslittle.me">
      <div class="stork-title"><p>Result Title</p></div>
      <div class="stork-excerpt-container">
        <div class="stork-excerpt">
          <p>
            ...This is the text of the excerpt....
          </p>
        </div>
      </div>
    </a>`
  );
});

test("resultToListItem with no excerpts doesn't have container", () => {
  const result: Result = {
    entry: {
      fields: {},
      title: "Result Title",
      url: "https://jameslittle.me"
    },
    excerpts: [],
    score: 12,
    title_highlight_ranges: []
  };

  const node = resultToListItem(result, { selected: false, showScores: false });
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  expect((node as Element).innerHTML).toEqualDisregardingWhitespace(
    `<a href="https://jameslittle.me">
      <div class="stork-title"><p>Result Title</p></div>
    </a>`
  );
});
