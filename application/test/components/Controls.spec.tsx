import React, { useReducer, useState } from 'react';
import Enzyme, { mount, ReactWrapper } from 'enzyme';
import Adapter from 'enzyme-adapter-react-16';
import { Controls } from '../../app/components/Controls';
import { mainReducer } from '../../app/reducers/reducer';
import { Dispatch, DispatchContext } from '../../app/actions/actions';
import { act } from 'react-dom/test-utils';
import { flushPromises } from '../helpers/tools';

import { GlobalContext, intialStore } from '../../app/store';

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
      isPackaged: jest.fn(),
      getVersion: jest.fn(() => 'test'),
    },
    dialog: {
      showOpenDialog: jest.fn(async () =>
        Promise.resolve({ filePaths: ['/test'] })
      ),
    },
  },
}));

function ControlsComponent() {
  const [store, rawDispatch] = useReducer(mainReducer, intialStore);
  const [dispatch] = useState(new Dispatch(rawDispatch));

  return (
    <GlobalContext.Provider value={store}>
      <DispatchContext.Provider value={dispatch}>
        <Controls handleLoad={() => {}} />
      </DispatchContext.Provider>
    </GlobalContext.Provider>
  );
}

const findAndClick = (component: ReactWrapper, target: string) => {
  component.find(target).first().simulate('click');
};

const findAndTest = (
  component: ReactWrapper,
  target: string,
  expected: string
) => {
  expect(component.find(target).first().text()).toBe(expected);
};

describe('Controls', () => {
  it('incremment editor type', () => {
    const component = mount(<ControlsComponent />);
    findAndClick(component, '#settingsButton');
    findAndTest(component, '#editorButton', 'Editor: Text');
    findAndClick(component, '#editorButton');
    findAndTest(component, '#editorButton', 'Editor: Vim');
    findAndClick(component, '#editorButton');
    findAndTest(component, '#editorButton', 'Editor: Emacs');
    findAndClick(component, '#editorButton');
    findAndTest(component, '#editorButton', 'Editor: Text');
    component.unmount();
  });

  it('updates working_path', async () => {
    const component = mount(<ControlsComponent />);
    findAndClick(component, '#settingsButton');
    await act(async () => {
      component.find('#workingPathButton').first().simulate('click');
      await flushPromises();
    });
    findAndTest(component, '#workingPathButton', 'Working Path: "/test"');
    component.unmount();
  });
});
