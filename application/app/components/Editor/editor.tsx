import React, { useState, useEffect, useContext } from 'react';
import AceEditor from 'react-ace';
import WSCMode from './mode.js';
import './theme';
import { GlobalContext, Editors } from '../../store';
import { DispatchContext } from '../../actions/actions';

import 'ace-builds/src-noconflict/mode-elixir';
import 'ace-builds/src-noconflict/theme-github';
import 'ace-builds/src-noconflict/keybinding-vim';
import 'ace-builds/src-noconflict/keybinding-emacs';
import 'ace-builds/src-noconflict/ext-language_tools';

const customMode = new WSCMode();

type Props = { handleLoad: () => void };

export const Editor = (props: Props): React.ReactElement => {
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);

  const [renderSpace, setRenderSpace] = useState<AceEditor | null>();
  const [render, setRender] = useState<boolean>(false);
  const [save, setSave] = useState<boolean>(false);

  useEffect(() => {
    if (renderSpace) {
      const getStoredLanguage = () => {
        const stored = localStorage.getItem('language');
        if (stored) {
          dispatch.onUpdateLanguage(stored);
        }
      };
      // @ts-ignore
      renderSpace.editor.getSession().setMode(customMode);
      renderSpace.editor.setTheme('ace/theme/wsc');
      getStoredLanguage();
    }
  }, [renderSpace, dispatch]);

  useEffect(() => {
    const submit = async () => {
      if (render) {
        await dispatch.onRender(store.language);
        setRender(false);
      }
    };

    submit().catch((e) => {
      throw e;
    });
  }, [render, dispatch, store.language]);

  useEffect(() => {
    if (save) {
      dispatch.onFileSave(store.language);
      setSave(false);
    }
  }, [store, dispatch, store.language, save]);

  useEffect(() => {
    if (store.initializeTest) {
      dispatch.onStop().catch((e) => {
        throw e;
      });
    }
  }, [dispatch, store.initializeTest]);

  return (
    <AceEditor
      focus={true}
      ref={(el) => {
        setRenderSpace(el);
      }}
      placeholder="WereSoCool"
      mode="elixir"
      theme="github"
      name="aceEditor"
      keyboardHandler={Editors[store.editor]['style']}
      value={store.language}
      onChange={(l) => dispatch.onUpdateLanguage(l)}
      markers={store.markers}
      fontSize={20}
      showPrintMargin={true}
      showGutter={true}
      highlightActiveLine={true}
      setOptions={{
        //enableLiveAutocompletion: true,
        enableBasicAutocompletion: true,
        showLineNumbers: true,
        tabSize: 2,
        displayIndentGuides: true,
      }}
      commands={[
        {
          name: 'submit',
          bindKey: { win: 'Shift-Enter', mac: 'Shift-Enter' },
          exec: () => {
            setRender(true);
          },
        },
        {
          name: 'stop',
          bindKey: { win: 'Command-p', mac: 'Command-Enter' },
          exec: async () => {
            await dispatch.onStop();
          },
        },
        {
          name: 'save',
          bindKey: { win: 'Ctrl-s', mac: 'Command-s' },
          exec: () => {
            setSave(true);
          },
        },
        {
          name: 'load',
          bindKey: { win: 'Ctrl-l', mac: 'Command-l' },
          exec: () => {
            props.handleLoad();
          },
        },
      ]}
      style={{
        height: '80vh',
        width: '80vw',
        marginLeft: '10vw',
      }}
    />
  );
};
