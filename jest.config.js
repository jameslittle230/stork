module.exports = {
  preset: "ts-jest",
  transform: {
    "^.+\\.jsx?$": "<rootDir>/node_modules/ts-jest/dist/index.js"
  },
  testEnvironment: "<rootDir>/js/test-environment.js",
  modulePathIgnorePatterns: ["<rootDir>/target/*"],
  collectCoverage: true,
  collectCoverageFrom: ["js/**/*.ts"],
  roots: ["js"]
};
