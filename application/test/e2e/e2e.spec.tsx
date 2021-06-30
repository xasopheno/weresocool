//  Important to match spectron and electron versions
//  https://github.com/electron-userland/spectron#version-map
import {Application} from 'spectron';
import {assert} from 'chai';
import {tutorial_list, album_list} from '../../app/components/tutorial_list';

describe('Application launch', function () {
  let app: Application;
  jest.setTimeout(50000);
  const app_path =
    process.platform === 'darwin'
      ? 'release/mac/WereSoCool.app/Contents/MacOS/WereSoCool'
      : 'release/linux-unpacked/weresocool';
  console.log('App Path:', app_path);

  beforeEach(function () {
    app = new Application({
      path: app_path,
      // args: [path.join(__dirname, '../../app')],
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
    const play_button = await app.client.$('#playButton');
    await play_button.click();
    //@ts-ignore
    const led_good = await app.client.$('#led_good');
    return led_good.isExisting();
  });

  it('should display #led_good after volume change', async function () {
    await app.client.waitUntilWindowLoaded();
    const volumeSlider = await app.client.$('#volumeSlider');
    await volumeSlider.setValue(50);
    const volumeText = await app.client.$('#volumeText');
    const led_good = await app.client.$('#led_good');
    const volume_value = await volumeText.getValue();
    return led_good.isExisting() && volume_value === '50';
  });

  it('should display #led_good after choosing each demo/tutorial', async function () {
    await app.client.waitUntilWindowLoaded();
    const results: {[key: string]: boolean} = {};
    const demos = [
      {button: '#magicButton', list: album_list},
      {button: '#questionButton', list: tutorial_list},
    ];
    for (const demo of demos) {
      for (const song in demo.list) {
        const demo_button = await app.client.$(demo.button);
        await demo_button.click();
        const song_name: string = demo.list[song].text;
        console.log(song_name);
        const search = `//*[text()[contains(., '${song_name}')]]`;
        // await app.client.scroll(search);
        const search_handle = await app.client.$(search);
        await search_handle.scrollIntoView();
        await search_handle.click();
        const led_good = await app.client.$('#led_good');
        const result = await led_good.isExisting();
        results[song_name] = result;
      }
    }
    console.log(results);
    for (const result in results) {
      if (results[result] !== true) {
        console.log('FAILURE------', result);
      }
    }
    const all_good = Object.values(results).every((v) => v === true);
    return assert.isTrue(all_good);
  });
});

// return app.client.debug();
