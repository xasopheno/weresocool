//  Important to match spectron and electron versions
//  https://github.com/electron-userland/spectron#version-map
//  const Application = require('spectron').Application;
//  const assert = require('assert');
import { Application } from 'spectron';
import { assert } from 'chai';
//  const tutorial_list = require('../../app/components/tutorial_list.tsx');
//  const electronPath = require('electron');
//  const path = require('path');

describe('Application launch', function () {
  let app: Application;
  jest.setTimeout(10000);

  beforeEach(function () {
    app = new Application({
      path: 'release/mac/WereSoCool.app/Contents/MacOS/WereSoCool',
      //  args: [path.join(__dirname, '../../app')],
      //  path: electronPath,
    });
    return app.start();
  });

  afterEach(function () {
    if (app && app.isRunning()) {
      return app.stop();
    } else {
      return Promise.resolve();
    }
  });

  it('shows an initial window', async function () {
    const count = await app.client.getWindowCount();
    return assert.equal(count, 1);
  });

  it('should display #led_good after render', async function () {
    await app.client.waitUntilWindowLoaded();
    //@ts-ignore
    await app.client.click('#playButton');
    //@ts-ignore
    return app.client.isExisting('#led_good');
  });

  // it.only('should display #led_good after choosing tutorial', async function () {
  // await this.app.client.waitUntilWindowLoaded();
  // for (tutorial in tutorial_list) {
  // console.log(tutorial);
  // }
  // await this.app.client.click('#magicButton');
  // return this.app.client.isExisting('#led_good');
  // });
});
