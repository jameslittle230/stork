const path = require("path");
const webpack = require("webpack");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "stork.js",
    library: "stork",
    chunkFilename: "storkmodule-[id].js"
  },
  plugins: [
    new CleanWebpackPlugin()
    // new CopyPlugin(
    //   [
    //     path.resolve(__dirname, "static"),
    //     {
    //       from: path.resolve(__dirname, "pkg", "stork_bg.wasm"),
    //       to: "stork.wasm"
    //     }
    //   ],
    //   { copyUnmodified: true }
    // )
  ],
  module: {
    rules: [
      {
        enforce: "pre",
        test: /\.js$/,
        exclude: /node_modules/,
        loader: "eslint-loader"
      },
      {
        test: /\.js$/,
        loader: require.resolve("@open-wc/webpack-import-meta-loader"),
        exclude: /(node_modules|bower_components)/
      }
    ]
  }
};
