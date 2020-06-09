import React from 'react';
import Enzyme, { mount } from 'enzyme';
import Adapter from 'enzyme-adapter-react-16';
import Root from '../../app/containers/Root';
import { testStore } from '../../app/store';
import { OuterSpaceWrapper } from '../helpers/wrappers';
import { act } from 'react-dom/test-utils';
import AceEditor from 'react-ace';
import { intialStore } from '../../app/store';
import { flushPromises } from '../helpers/tools';
import FileSaver from 'file-saver';

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

describe('OuterSpace', () => {
  it('onResetLanguage', () => {
    const component = mount(<OuterSpaceWrapper />);
    // @ts-ignore
    const editor = component.find(AceEditor).instance().editor;
    act(() => {
      editor.setValue('code');
    });
    expect(editor.getValue()).toBe('code');
    act(() => {
      component.find('#resetButton').at(0).simulate('click');
    });
    component.update();
    expect(editor.getValue()).toBe(intialStore.language);
  });
  //
  it('onFileSave', async () => {
    const component = mount(<OuterSpaceWrapper />);
    FileSaver.saveAs = jest.fn();
    await act(async () => {
      const saveButton = component.find('#saveButton');
      saveButton.at(0).simulate('click');

      await flushPromises();
    });

    expect(FileSaver.saveAs).toHaveBeenCalled();
  });

  it('onFileLoad', async () => {
    const component = mount(<OuterSpaceWrapper />);
    const expected = 'language from file';
    const blob = new Blob([expected], { type: '.socool' });
    await act(async () => {
      const loadInput = component.find('#fileLoadInput');
      loadInput.at(0).simulate('change', { target: { files: [blob] } });

      await flushPromises();
    });
    const editor = component
      .find(AceEditor)
      // @ts-ignore
      .instance().editor;
    expect(editor.getValue()).toBe(expected);
  });

  it('displays title', () => {
    const component = mount(<Root initialStore={testStore} />);
    expect(component.find('#outerSpace').exists()).toBe(true);
  });

  it('displays ratios only when wide', () => {
    window = Object.assign(window, { innerWidth: 500 });
    let component = mount(<Root initialStore={testStore} />);
    expect(component.find('#ratios').exists()).toBe(false);
    window = Object.assign(window, { innerWidth: 1500 });
    component = mount(<Root initialStore={testStore} />);
    expect(component.find('#ratios').exists()).toBe(true);
  });
});
