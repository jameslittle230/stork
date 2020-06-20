const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  resolve: {
    extensions: [".ts", ".tsx", ".js"]
  },
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "stork.js",
    library: "stork"
  },
  devtool: "inline-source-map",
  plugins: [
    new CleanWebpackPlugin(),
    new CopyPlugin(
      [
        path.resolve(__dirname, "dist"),
        {
          from: path.resolve(__dirname, "pkg", "stork_bg.wasm"),
          to: "stork.wasm"
        },
        {
          from: path.resolve(__dirname, "test/static", "*"),
          to: ".",
          flatten: true
        },
        {
          from: path.resolve(__dirname, "test", "3b1b.st"),
          to: ".",
          flatten: true
        },
        {
          from: path.resolve(__dirname, "test", "federalist.st"),
          to: ".",
          flatten: true
        }
      ],
      { copyUnmodified: true }
    )
  ],
  module: {
    rules: [
      { test: /\.ts?$/, loader: "ts-loader" },
      { test: /\.js$/, loader: "source-map-loader" },
      {
        test: /\.js$/,
        loader: require.resolve("@open-wc/webpack-import-meta-loader"),
        exclude: /(node_modules|bower_components)/
      }
    ]
  }
};
