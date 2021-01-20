const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const { version } = require("./package.json");
const { DefinePlugin } = require("webpack");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  resolve: {
    extensions: [".ts", ".tsx", ".js"]
  },
  entry: {
    index: "./js/main.ts"
  },
  output: {
    path: dist,
    filename: "stork.js",
    library: "stork"
  },
  devtool: "inline-source-map",
  plugins: [
    new DefinePlugin({
      "process.env.VERSION": JSON.stringify(version)
    }),
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
      { test: /\.ts?$/, loader: "ts-loader" },
      { test: /\.js$/, loader: "source-map-loader" },
      {
        test: /\.js$/,
        loader: require.resolve("@open-wc/webpack-import-meta-loader")
      }
    ]
  }
};
