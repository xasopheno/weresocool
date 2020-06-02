//  Important to match spectron and electron versions
//  https://github.com/electron-userland/spectron#version-map
const Application = require('spectron').Application;
const assert = require('assert');
const electronPath = require('electron');
const path = require('path');

describe('Application launch', function () {
  this.timeout(10000);

  beforeEach(function () {
    this.app = new Application({
      path: electronPath,
      //  path: 'release/mac/WereSoCool.app/Contents/MacOS/WereSoCool',
      args: [path.join(__dirname, '../../app')],
    });
    return this.app.start();
  });

  afterEach(function () {
    if (this.app && this.app.isRunning()) {
      return this.app.stop();
    }
  });

  it('shows an initial window', async function () {
    const count = await this.app.client.getWindowCount();
    return assert.equal(count, 1);
  });

  it('should display #led_good after render', async function () {
    await this.app.client.waitUntilWindowLoaded();
    await this.app.client.click('#renderButton');
    return this.app.client.isExisting('#led_good');
  });
});
