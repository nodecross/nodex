module.exports = {
  "transform": {
    "^.+\\.jsx?$": "babel-jest",
    "^.+\\.tsx?$": "ts-jest"
  },
  verbose: true,
  "roots": [
    "<rootDir>/__tests__",
  ],
  "testMatch": [
      "**/__tests__/**/*.+(ts|tsx|js)",
      "**/?(*.)+(spec|test).+(ts|tsx|js)"
  ],
  moduleFileExtensions: ['js', 'ts']
}