import React, { useReducer, useState } from 'react';
import { Switch, Route } from 'react-router-dom';
import { OuterSpace } from './containers/outerSpace';
import { GlobalContext, Store } from './store';
import { DispatchContext, Dispatch } from './actions/actions';
import { mainReducer } from './reducers/reducer';

type Props = { initialStore: Store };

export default function Routes(props: Props): React.ReactElement {
  const [store, rawDispatch] = useReducer(mainReducer, props.initialStore);
  const [dispatch] = useState(new Dispatch(rawDispatch));

  return (
    <GlobalContext.Provider value={store}>
      <DispatchContext.Provider value={dispatch}>
        <Switch>
          <Route path={'/'}>
            <OuterSpace />
          </Route>
        </Switch>
      </DispatchContext.Provider>
    </GlobalContext.Provider>
  );
}
