const path = require("path");
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
  plugins: [new CleanWebpackPlugin()],
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
