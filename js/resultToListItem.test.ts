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
      <div style="display: flex; justify-content: space-between">
        <p class="stork-title">Result Title</p>
      </div>
      <div style="display: flex; justify-content: space-between">
        <p class="stork-excerpt">
          ...This is the text of the excerpt....
        </p>
      </div>
    </a>`
  );
});
