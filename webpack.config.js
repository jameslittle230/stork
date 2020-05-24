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
          from: path.resolve(__dirname, "test", "*.st"),
          to: ".",
          flatten: true
        }
      ],
      { copyUnmodified: true }
    )
  ],
  module: {
    rules: [
      { test: /\.tsx?$/, loader: "awesome-typescript-loader" },
      { test: /\.js$/, loader: "source-map-loader" },
      // {
      //   enforce: "pre",
      //   test: /\.js$/,
      //   exclude: /node_modules/,
      //   loader: "eslint-loader"
      // },
      {
        test: /\.js$/,
        loader: require.resolve("@open-wc/webpack-import-meta-loader"),
        exclude: /(node_modules|bower_components)/
      }
    ]
  }
};
