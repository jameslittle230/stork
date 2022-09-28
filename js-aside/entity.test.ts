import { Entity } from "./entity";
import { defaultConfig } from "./config";
jest.mock("./wasmQueue");
jest.mock("./entityDom");
// eslint-disable-next-line @typescript-eslint/no-empty-function
jest.mock("stork-search", () => {}, { virtual: true });

test("Can successfully generate an entity", () => {
  const entity = new Entity("test", "https://google.com", defaultConfig);
  expect(entity).toBeTruthy();
});

test("Injest search data maps url values", () => {
  const entity = new Entity("test", "https://google.com", defaultConfig);
  entity.injestSearchData({
    results: [
      {
        entry: { fields: {}, title: "bleh", url: "https://google.com" },
        excerpts: [
          {
            fields: {},
            internal_annotations: [{ a: "#suffix" }],
            highlight_ranges: [],
            score: 0,
            text: "blah"
          }
        ],
        score: 0,
        title_highlight_ranges: []
      }
    ],
    total_hit_count: 0,
    url_prefix: ""
  });

  expect(entity.results[0].entry.url).toEqual("https://google.com#suffix");
});

test("Changing an entity's state calls render", () => {
  const entity = new Entity("test", "https://google.com", defaultConfig);
  entity.attachToDom();
  entity.state = "loading";
  expect(entity.domManager?.render as jest.Mock).toHaveBeenCalled();
});

test("Set download progress should render only if the entity's config shows the progress", () => {
  const entities = [false, true].map(showProgress => {
    const entity = new Entity("test", "https://google.com", {
      ...defaultConfig,
      showProgress
    });
    entity.attachToDom();
    entity.setDownloadProgress(20);
    return entity;
  });

  const [e1_render_calls, e2_render_calls] = entities.map(
    e => (e.domManager?.render as jest.Mock).mock.calls.length
  );

  // Entity 2's domManager has one more render call than entity 1's.
  expect(e2_render_calls - e1_render_calls).toEqual(1);
});

test("Errored download calls render with an error", () => {
  const entity = new Entity("test", "https://google.com", defaultConfig);
  entity.attachToDom();
  entity.setDownloadError();

  const lastCall = (entity.domManager?.render as jest.Mock).mock.calls[1][0];
  console.log(lastCall);
  expect(lastCall.state).toEqual("error");
  expect(lastCall.message.toLowerCase()).toContain("error");
});
