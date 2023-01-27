// eslint-disable-next-line no-undef
module.exports = {
  env: {
    browser: true,
    es6: true
  },
  extends: ["eslint:recommended", "plugin:@typescript-eslint/recommended"],
  parser: "@typescript-eslint/parser",
  plugins: ["@typescript-eslint"],
  root: false,
  // extends: ["plugin:prettier/recommended"],
  // globals: {
  //   Atomics: "readonly",
  //   SharedArrayBuffer: "readonly"
  // },
  parserOptions: {
    ecmaVersion: 2018,
    sourceType: "module"
  },
  rules: {
    "no-unused-vars": "off",
    //   "no-empty-function": "off",
    //   "@typescript-eslint/no-empty-function": "off",
    "@typescript-eslint/no-unused-vars": [
      "warn", // or "error"
      {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_",
        caughtErrorsIgnorePattern: "^_"
      }
    ]
    //   "prettier/prettier": [
    //     "error",
    //     {},
    //     {
    //       usePrettierrc: true
    //     }
    //   ]
  }
};

// TODO: Use the sort imports prettier plugin
