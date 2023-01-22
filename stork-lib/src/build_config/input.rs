#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use super::{html::HTMLConfig, File, FrontmatterConfig, SRTConfig, StemmingConfig};

#[derive(Serialize, Deserialize, Clone, Debug, SmartDefault, PartialEq, Eq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub enum TitleBoost {
    Minimal,
    #[default]
    Moderate,
    Large,
    Ridiculous,
}

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
#[allow(non_snake_case)]
pub struct InputConfig {
    // If Stork is indexing files on your filesystem, this is the base directory
    // that should be used to resolve relative paths. This path will be in
    // relation to the working directory when you run the `stork build` command.
    pub base_directory: String,

    // Each file has a target URL to which it links. If all those target URLs
    // have the same prefix, you can set that prefix here to make shorter file objects.
    pub url_prefix: String,

    // The list of documents Stork should index.
    pub files: Vec<File>,

    // Determines how much a result will be boosted if the search query
    // matches the title.
    pub title_boost: TitleBoost,

    // The stemming algorithm the indexer should use while analyzing words.
    // Should be `None` or one of the languages supported by Snowball Stem,
    // e.g. `Dutch`.
    #[serde(default)]
    pub stemming: StemmingConfig,

    #[serde(default)]
    pub html_config: HTMLConfig,

    // If frontmatter is detected in your content, `Ignore` will not handle the
    // frontmatter in any special way, effectively including the raw text in
    // the index. `Omit` will parse and remove frontmatter from indexed content.
    // `Parse` will include frontmatter key/value pairs in the file's `fields`
    // property, except for the keys `title` and `url` which will set the file's
    // title and url, respectively. When a title, url, or field is specified in
    // frontmatter and in the Stork config, the value in the config takes precedence.
    #[serde(default)]
    pub frontmatter_config: FrontmatterConfig,

    // For all SRT files, this object will describe how Stork will handle the
    // timestamp information embedded in the file.
    #[serde(default)]
    pub srt_config: SRTConfig,
}
