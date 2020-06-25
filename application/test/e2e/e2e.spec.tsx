//  Important to match spectron and electron versions
//  https://github.com/electron-userland/spectron#version-map
import { Application } from 'spectron';
import { assert } from 'chai';
import { tutorial_list, album_list } from '../../app/components/tutorial_list';

describe('Application launch', function () {
  let app: Application;
  jest.setTimeout(50000);

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

  it('should display #led_good after choosing each demo/tutorial', async function () {
    await app.client.waitUntilWindowLoaded();
    const results: { [key: string]: boolean } = {};
    const demos = [
      { button: '#magicButton', list: album_list },
      { button: '#questionButton', list: tutorial_list },
    ];
    for (const demo of demos) {
      for (const song in demo.list) {
        //@ts-ignore
        await app.client.click(demo.button);
        const song_name: string = demo.list[song].text;
        console.log(song_name);
        const search = `//*[text()[contains(., '${song_name}')]]`;
        //@ts-ignore
        await app.client.scroll(search);
        //@ts-ignore
        await app.client.element(search).click();
        //@ts-ignore
        const result: boolean = await app.client.isExisting('#led_good');
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
