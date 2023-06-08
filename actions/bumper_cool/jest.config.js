module.exports = {
  verbose: true, // Each individual test will be reported during the run
  collectCoverage: true, // Collect code coverage by Istanbul
  testEnvironment: 'node', // Making sure we use the Node environment
  moduleFileExtensions: ['js', 'json', 'ts'], // The file extensions Jest will accept
  testMatch: ['**/*.spec.(js|ts)|**/*.spec.(js|ts)'], // The glob patterns Jest uses to detect test files
  transform: {
    '^.+\\.(t|j)s$': 'ts-jest', // Using ts-jest for TypeScript and JavaScript files
  },
}
