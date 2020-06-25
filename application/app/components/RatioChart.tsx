import React from 'react';
import styled from 'styled-components';

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

export const RatioChart = (): React.ReactElement => {
  return (
    <div>
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
    </div>
  );
};
