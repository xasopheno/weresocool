import React, { useContext, useState, useEffect } from 'react';
import { GlobalContext } from '../store';

import styled from 'styled-components';
import { DispatchContext } from '../actions/actions';
import path from 'path';
const remote = require('electron').remote;
const fs = remote.require('fs');

const RSpace = styled.div`
  position: absolute;
  top: 10%;
  right: 0;
  display: flex;
  flex-direction: column;
  font-family: 'Courier New', Courier, monospace;
  font-size: 1.1em;
  margin-right: 1em;
  text-align: center;
  border: 5px ridge goldenrod;
`;

const Maj = styled.div`
  font-size: 1.1em;
  font-weight: bold;
  padding-left: 0.25em;
  padding-right: 0.25em;
  border: 2px ridge darkgoldenrod;
`;

const Degree = styled.div`
  padding-top: 0.5em;
  padding-bottom: 0.5em;
`;

const Thirteen = styled.p`
  color: deeppink;
  padding: 0 0 0 0;
  margin: 0 0 0 0;
`;

const Eleven = styled.p`
  color: salmon;
  padding: 0 0 0 0;
  margin: 0 0 0 0;
`;

const Seven = styled.p`
  color: #7fcdcd;
  padding: 0 0 0 0;
  margin: 0 0 0 0;
`;

const Five = styled.p`
  color: lightpink;
  padding: 0 0 0 0;
  margin: 0 0 0 0;
`;

const Three = styled.p`
  color: #f9d976;
  padding: 0 0 0 0;
  margin: 0 0 0 0;
`;

const Two = styled.p`
  color: gold;
  padding: 0 0 0 0;
  margin: 0 0 0 0;
`;
const MagicButton = styled.img`
  width: 70px;
  height: 70px;
  border-top: 5px ridge goldenrod;
  margin-top: 10px;
`;

export const Ratios = (props: { width: number }): React.ReactElement | null => {
  if (props.width > 1000) {
    return <RatiosInner />;
  } else {
    return null;
  }
};

export const RatiosInner = (): React.ReactElement => {
  const dispatch = useContext(DispatchContext);
  const store = useContext(GlobalContext);
  const [render, setRender] = useState<boolean>(false);

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

  const assetsPath = remote.app.isPackaged
    ? path.join(process.resourcesPath, 'extraResources/assets')
    : 'assets';

  const demoSong = () => {
    const home = `${remote.app.getPath('home')}/Documents/weresocool/demo`;

    const songs: Array<string> = [];
    try {
      const files = fs.readdirSync(home);
      for (const i in files) {
        songs.push(files[i]);
      }
    } catch (e) {
      console.log(e);
    }

    const song = songs[Math.floor(Math.random() * songs.length)];
    console.log(song);

    fs.readFile(`${home}/${song}`, 'utf-8', function read(
      err: Error,
      data: string
    ) {
      if (err) {
        throw err;
      }

      dispatch.onUpdateLanguage(data);
      setRender(true);
    });
  };

  return (
    <RSpace id="ratios">
      <Degree>
        <Maj>
          <Two>2/1</Two>
        </Maj>
      </Degree>

      <Degree>
        <Thirteen>25/13</Thirteen>
        <Maj>
          <Five>15/8</Five>
        </Maj>
        <Seven>7/4</Seven>
      </Degree>

      <Degree>
        <Seven>12/7</Seven>
        <Maj>
          <Five>5/3</Five>
        </Maj>
        <Thirteen>13/8</Thirteen>
        <Five>8/5</Five>
        <Seven>14/9</Seven>
      </Degree>

      <Degree>
        <Maj>
          <Three>3/2</Three>
        </Maj>
      </Degree>
      <Degree>
        <Seven>10/7</Seven>
        <Seven>7/5</Seven>
        <Eleven>11/8</Eleven>
      </Degree>
      <Degree>
        <Maj>
          <Three>4/3</Three>
        </Maj>
      </Degree>
      <Degree>
        <Seven>9/7</Seven>
        <Maj>
          <Five>5/4</Five>
        </Maj>
        <Five>6/5</Five>
        <Seven>7/6</Seven>
      </Degree>

      <Degree>
        <Thirteen>15/13</Thirteen>
        <Seven>8/7</Seven>
        <Maj>
          <Three>9/8</Three>
        </Maj>
        <Five>25/24</Five>
      </Degree>

      <Degree>
        <Maj>
          <Two>1/1</Two>
        </Maj>
      </Degree>
      <MagicButton src={`${assetsPath}/magic.png`} onClick={() => demoSong()} />
    </RSpace>
  );
};
