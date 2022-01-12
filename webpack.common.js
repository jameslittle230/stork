const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const { version } = require("./package.json");
const { DefinePlugin } = require("webpack");

module.exports = {
  resolve: {
    extensions: [".ts", ".tsx", ".js"]
  },
  entry: {
    index: "./js/main.ts"
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "stork.js",
    library: "stork"
  },
  experiments: {
    asyncWebAssembly: true
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
          from: path.resolve(__dirname, "stork-wasm", "pkg", "stork_bg.wasm"),
          to: "stork.wasm"
        }
      ],
      { copyUnmodified: true }
    )
  ],
  module: {
    rules: [
      { test: /\.ts?$/, loader: "ts-loader" },
      { test: /\.js$/, loader: "source-map-loader", sideEffects: true },
      {
        test: /\.js$/,
        loader: require.resolve("@open-wc/webpack-import-meta-loader")
      }
    ]
  }
};
