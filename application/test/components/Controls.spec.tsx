import React, { useReducer, useState } from 'react';
import Enzyme, { mount, ReactWrapper } from 'enzyme';
import Adapter from 'enzyme-adapter-react-16';
import { Controls } from '../../app/components/Controls';
import { mainReducer } from '../../app/reducers/reducer';
import { Dispatch, DispatchContext } from '../../app/actions/actions';

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
    },
  },
  dialog: jest.fn(),
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
    // findAndTest(component, '#editorButton', 'Text');
    // findAndClick(component, '#editorButton');
    findAndTest(component, '#editorButton', 'Vim');
    findAndClick(component, '#editorButton');
    findAndTest(component, '#editorButton', 'Emacs');
    findAndClick(component, '#editorButton');
    findAndTest(component, '#editorButton', 'Text');
    component.unmount();
  });
});
