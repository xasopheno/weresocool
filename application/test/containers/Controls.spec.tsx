import React from 'react';
import Enzyme, { mount } from 'enzyme';
import { testStore } from '../../app/store';
import Adapter from 'enzyme-adapter-react-16';
import Root from '../../app/containers/Root';
import axios from 'axios';
import MockAdapter from 'axios-mock-adapter';
import { act } from 'react-dom/test-utils';
import { OuterSpaceWrapper } from '../helpers/wrappers';
import { flushPromises } from '../helpers/tools';

Enzyme.configure({ adapter: new Adapter() });

describe('Render', () => {
  test('click #render: ParseError', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      ParseError: { message: 'Unexpected Token', line: 14, column: 15 },
    };
    mock.onPost().reply(200, response);

    const component = mount(<OuterSpaceWrapper />);
    await act(async () => {
      component.find('#renderButton').at(0).simulate('click');
      await flushPromises();
    });
    component.update();

    const errorDescription = component.find('#errorDescription');
    expect(errorDescription.exists()).toBe(true);
    expect(errorDescription.at(0).text()).toBe(
      'UnexpectedToken: Line: 14 | Column 15'
    );
  });
  test('click #render: RenderSuccess', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      RenderSuccess: 'success',
    };
    mock.onPost().reply(200, response);

    const component = mount(<OuterSpaceWrapper />);

    await act(async () => {
      component.find('#renderButton').at(0).simulate('click');
      await flushPromises();
    });
    component.update();

    const errorDescription = component.find('#errorDescription');
    expect(errorDescription.exists()).toBe(false);
  });
  test('click #render: IdError', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      IdError: { id: 'thing' },
    };
    mock.onPost().reply(200, response);

    const component = mount(<OuterSpaceWrapper />);
    await act(async () => {
      component.find('#renderButton').at(0).simulate('click');
      await flushPromises();
    });
    component.update();

    const errorDescription = component.find('#errorDescription');
    expect(errorDescription.exists()).toBe(true);
    expect(errorDescription.at(0).text()).toBe('Name Not Found: thing');
  });
  test('click #render: IndexError', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      IndexError: {
        len_list: 7,
        index: 8,
        message: 'index 8 is greater than length of list 7',
      },
    };
    mock.onPost().reply(200, response);

    const component = mount(<OuterSpaceWrapper />);
    await act(async () => {
      component.find('#renderButton').at(0).simulate('click');
      await flushPromises();
    });
    component.update();

    const errorDescription = component.find('#errorDescription');
    expect(errorDescription.exists()).toBe(true);
    expect(errorDescription.at(0).text()).toBe(
      'index 8 is greater than length of list 7'
    );
  });
  test('click #render: MsgError', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      Msg: {
        message: 'I am a message',
      },
    };
    mock.onPost().reply(200, response);

    const component = mount(<OuterSpaceWrapper />);
    await act(async () => {
      component.find('#renderButton').at(0).simulate('click');
      await flushPromises();
    });
    component.update();

    const errorDescription = component.find('#errorDescription');
    expect(errorDescription.exists()).toBe(true);
    expect(errorDescription.at(0).text()).toBe('Error: I am a message');
  });
});

describe('Controls', () => {
  it('render button exists', () => {
    const component = mount(<Root initialStore={testStore} />);
    expect(component.find('#renderButton').exists()).toBe(true);
  });
  it('stop button exists', () => {
    const component = mount(<Root initialStore={testStore} />);
    expect(component.find('#stopButton').exists()).toBe(true);
  });
  it('load button exists', () => {
    const component = mount(<Root initialStore={testStore} />);
    expect(component.find('#loadButton').exists()).toBe(true);
  });
  it('save button exists', () => {
    const component = mount(<Root initialStore={testStore} />);
    expect(component.find('#saveButton').exists()).toBe(true);
  });
  it('reset button exists', () => {
    const component = mount(<Root initialStore={testStore} />);
    expect(component.find('#resetButton').exists()).toBe(true);
  });
  it('editor button exists', () => {
    const component = mount(<Root initialStore={testStore} />);
    expect(component.find('#editorButton').exists()).toBe(true);
  });
});
