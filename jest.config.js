module.exports = {
  preset: "ts-jest",
  testEnvironment: "node",
  modulePathIgnorePatterns: ["<rootDir>/target/*", "<rootDir>/pkg/*"],
  collectCoverageFrom: ["js/**/*.{js,ts}"]
};
