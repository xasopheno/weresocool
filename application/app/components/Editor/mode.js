import 'ace-builds/src-noconflict/mode-text';

export class CustomHighlightRules extends window.ace.acequire(
  'ace/mode/text_highlight_rules'
).TextHighlightRules {
  constructor() {
    super();
    this.$rules = {
      start: [
        {
          token: ['hash_tag', 'name_tag'],
          regex: '(#)([a-z_0-9]+[A-Z]*)',
        },
        {
          token: [
            'stem_arrow',
            '',
            'stem_bracket',
            'stem_names',
            'stem_bracket',
          ],
          regex: '(->)(\\s?)(\\[)(.*)(\\])',
        },
        {
          token: 'comment',
          regex: '--.*$',
        },
        {
          token: 'number',
          regex: '[1-9]',
        },
        {
          token: 'zero',
          regex: '[0]',
        },
        //  {
        //  token: 'danny',
        //  regex: 'f:|l:|g:|p:',
        //  },
        {
          token: 'slash',
          regex: '/',
        },
        {
          token: 'lambda',
          regex: '\\\\|Lambda',
        },
        {
          token: 'keyword',
          regex: '#',
        },
        {
          token: 'curly',
          regex: '{|}',
        },
        {
          token: 'bracket',
          regex: '\\[|\\]',
        },
        {
          token: 'paren',
          regex: '\\(|\\)',
        },
        {
          token: 'pipe',
          regex: '\\|',
        },

        {
          token: 'keyword',
          regex: '>',
        },

        {
          token: 'curly',
          regex: '=',
        },
        {
          token: 'curly',
          regex: 'main',
        },
        {
          token: 'import',
          regex: 'import',
        },
        {
          token: 'dot',
          regex: '\\.',
        },
        {
          token: 'group_operation_other',
          regex: 'FitLength|ModulateBy|Reverse|ModBy|Invert',
        },
        {
          token: 'repeat',
          regex: 'Repeat',
        },
        {
          token: 'reverb',
          regex: 'Reverb',
        },
        { token: 'group_operation', regex: 'Sequence|Overlay|Seq' },
        {
          token: 'o_shortcut',
          regex: 'O',
        },
        {
          token: 'operation',
          regex: 'Sine|Square|Noise|Portamento|Pulse|Triangle|Tri',
        },
        {
          token: 'frequency',
          regex: 'AsIs|Fm|Fa|f:',
        },
        {
          token: 'length',
          regex: 'Lm|Length|l:',
        },
        {
          token: 'pan',
          regex: 'PanM|PanA|Pm|Pa|p:',
        },
        {
          token: 'gain',
          regex: 'Gain|Gm|g:',
        },
        {
          token: 'list',
          regex: '@|List|Random|&|ET',
        },
        {
          token: 'generator',
          regex: '\\*|Gen|Take',
        },
        {
          token: 'gen_part',
          regex: 'Poly|Expr',
        },
        {
          token: ['expr_quote', 'expr', 'expr_quote'],
          regex: '(`)(.*)(`)',
        },
        {
          token: 'letter',
          regex: '[a-z]',
        },
      ],
    };
  }
}

export default class WSCMode extends window.ace.acequire('ace/mode/text').Mode {
  constructor() {
    super();
    this.HighlightRules = CustomHighlightRules;
    this.lineCommentStart = '--';

    this.getNextLineIndent = function (state, line, tab) {
      var indent = this.$getIndent(line);

      var tokenizedLine = this.getTokenizer().getLineTokens(line, state);
      var tokens = tokenizedLine.tokens;

      if (tokens.length && tokens[tokens.length - 1].type === 'comment') {
        return indent;
      }

      if (state === 'start') {
        var match = line.match(/^.*[{([]\s*$/);
        if (match) {
          indent += tab;
        }
      }

      return indent;
    };

    //  this.checkOutdent = function (state, line, input) {
    //  return this.$outdent.checkOutdent(line, input);
    //  };

    //  this.autoOutdent = function (state, doc, row) {
    //  this.$outdent.autoOutdent(doc, row);
    //  };
  }
}
