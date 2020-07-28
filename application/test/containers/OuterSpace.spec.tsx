import React from 'react';
import Enzyme, { mount, ReactWrapper } from 'enzyme';
import Adapter from 'enzyme-adapter-react-16';
import Root from '../../app/containers/Root';
import { testStore } from '../../app/store';
import { OuterSpaceWrapper } from '../helpers/wrappers';
import { act } from 'react-dom/test-utils';
import AceEditor from 'react-ace';
import { language_template } from '../../app/components/Editor/language_template';
import { flushPromises } from '../helpers/tools';
import FileSaver from 'file-saver';
import MockAdapter from 'axios-mock-adapter';
import axios from 'axios';

Enzyme.configure({ adapter: new Adapter() });
// @ts-ignore
process.resourcesPath = 'test';
jest.mock('electron', () => ({
  require: jest.fn(),
  match: jest.fn(),
  app: jest.fn(),
  remote: {
    app: {
      getPath: jest.fn(),
      getVersion: jest.fn(() => 'test'),
      isPackaged: jest.fn(),
    },
  },
  dialog: jest.fn(),
}));

const click = async (button: string, component: Enzyme.ReactWrapper) => {
  await act(async () => {
    component.find(button).at(0).simulate('click');
    await flushPromises();
  });
};

describe('OuterSpace', () => {
  it('onUpdateVolume', async () => {
    let component: ReactWrapper;
    await act(async () => {
      component = mount(<OuterSpaceWrapper />);
      await flushPromises();
    });
    // @ts-ignore
    await act(async () => {
      component.find('volumeSlider').at(0).simulate('click');
      await flushPromises();
    });

    // act(() => {
    // editor.setValue('code');
    // });
    // expect(editor.getValue()).toBe('code');
    // act(() => {
    // component.find('#resetButton').at(0).simulate('click');
    // component.update();
    // });
    // expect(editor.getValue()).toBe(language_template);
    // expect(editor.focus.mock.calls.length).toBe(1);
  });
  it('onResetLanguage', async () => {
    let component: ReactWrapper;
    await act(async () => {
      component = mount(<OuterSpaceWrapper />);
      await flushPromises();
    });
    // @ts-ignore
    const editor = component.find(AceEditor).instance().editor;
    editor.focus = jest.fn();

    act(() => {
      editor.setValue('code');
    });
    expect(editor.getValue()).toBe('code');
    act(() => {
      component.find('#resetButton').at(0).simulate('click');
      component.update();
    });
    expect(editor.getValue()).toBe(language_template);
    expect(editor.focus.mock.calls.length).toBe(1);
  });

  describe('Render', () => {
    test('click #Render', async () => {
      const mock = new MockAdapter(axios);
      const response = {
        PrintSuccess: { audio: [0.0], print_type: 'wav' },
      };
      mock.onPost().reply(200, response);
      FileSaver.saveAs = jest.fn();

      for (const filetype of ['wav', 'mp3']) {
        const component = mount(<OuterSpaceWrapper />);
        expect(component.find('#renderModal').exists()).toBe(false);

        await click('#printButton', component);
        component.update();
        expect(component.find('#renderModal').exists()).toBe(true);
        await click(`#${filetype}Button`, component);

        expect(FileSaver.saveAs).toHaveBeenCalled();
      }
    });
  });
  it('onFileSave', async () => {
    const component = mount(<OuterSpaceWrapper />);
    FileSaver.saveAs = jest.fn();

    const editor = component
      .find(AceEditor)
      // @ts-ignore
      .instance().editor;
    editor.focus = jest.fn();

    await click('#saveButton', component);

    expect(FileSaver.saveAs).toHaveBeenCalled();
    expect(editor.focus.mock.calls.length).toBe(1);
  });

  it('onFileLoad', async () => {
    let component: ReactWrapper;
    await act(async () => {
      component = mount(<OuterSpaceWrapper />);
      await flushPromises();
    });

    const expected = 'language from file';
    const blob = new Blob([expected], { type: '.socool' });

    // @ts-ignore
    const editor = component
      .find(AceEditor)
      // @ts-ignore
      .instance().editor;
    editor.focus = jest.fn();
    // });

    await act(async () => {
      const loadInput = component.find('#fileLoadInput');
      loadInput.at(0).simulate('change', { target: { files: [blob] } });

      await flushPromises();
    });

    expect(editor.getValue()).toBe(expected);
    expect(editor.focus.mock.calls.length).toBe(1);
  });

  it('displays title', async () => {
    await act(async () => {
      const component = mount(<Root initialStore={testStore} />);
      await flushPromises();
      expect(component.find('#outerSpace').exists()).toBe(true);
    });
  });

  it('displays ratios only when wide', async () => {
    window = Object.assign(window, { innerWidth: 500 });
    let component: ReactWrapper;
    await act(async () => {
      component = mount(<Root initialStore={testStore} />);
      await flushPromises();
      expect(component.find('#ratios').exists()).toBe(false);
      window = Object.assign(window, { innerWidth: 1500 });
      component = mount(<Root initialStore={testStore} />);
      expect(component.find('#ratios').exists()).toBe(true);
    });
  });
});
