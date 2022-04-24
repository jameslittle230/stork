import { highlight } from "./pencil";
import { Result } from "./searchData";

export interface ListItemDisplayOptions {
  selected: boolean;
  showScores: boolean;
}

export function resultToListItem(
  result: Result,
  options: ListItemDisplayOptions
): ChildNode {
  const template = document.createElement("template");
  template.innerHTML = `
<li class="stork-result${options.selected ? " selected" : ""}">
  <a href="${result.entry.url}">
    <div class="stork-title">
      <p>${highlight(
        result.entry.title,
        result.title_highlight_ranges || []
      )}</p>
      ${options.showScores ? `<code><b>${result.score}</b></code>` : ""}
    </div>
    ${
      result.excerpts.length > 0
        ? '<div class="stork-excerpt-container" />'
        : ""
    }
    ${result.excerpts
      .map(
        e => `<div class="stork-excerpt"><p>
        ...${highlight(e.text, e.highlight_ranges || [])}...
        </p>
        ${options.showScores ? `<code>${e.score}</code>` : ""}
        </div>`
      )
      .join("")}
        ${result.excerpts.length > 0 ? "</div>" : ""}
  </a>
</li>`;
  return template.content.firstElementChild as ChildNode;
}
