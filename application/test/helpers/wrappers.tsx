import React, { useState, useReducer } from 'react';
import { GlobalContext, testStore } from '../../app/store';
import { DispatchContext, Dispatch } from '../../app/actions/actions';
import { mainReducer } from '../../app/reducers/reducer';
import { OuterSpace } from '../../app/containers/OuterSpace';

export const OuterSpaceWrapper = (): React.ReactElement => {
  const [store, rawDispatch] = useReducer(mainReducer, testStore);
  const [dispatch] = useState(new Dispatch(rawDispatch));

  return (
    <GlobalContext.Provider value={store}>
      <DispatchContext.Provider value={dispatch}>
        <OuterSpace />
      </DispatchContext.Provider>
    </GlobalContext.Provider>
  );
};
