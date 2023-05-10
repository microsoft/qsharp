import {
  JupyterFrontEnd,
  JupyterFrontEndPlugin
} from '@jupyterlab/application';

import { ICommandPalette } from '@jupyterlab/apputils';
import * as codemirror from '@jupyterlab/codemirror';
import * as notebook from '@jupyterlab/notebook';

/**
 * Initialization data for the qsharp_jupyterlab extension.
 */
const plugin: JupyterFrontEndPlugin<void> = {
  id: 'jupyterlab-apod',
  autoStart: true,
  requires: [
    ICommandPalette,
    codemirror.ICodeMirror,
    notebook.INotebookTracker
  ],
  activate: async (
    app: JupyterFrontEnd,
    palette: ICommandPalette,
    codemirror: codemirror.ICodeMirror,
    notebookTracker: notebook.INotebookTracker
  ) => {
    let rules = [
      {
        token: 'comment',
        regex: /(\/\/).*/,
        beginWord: false
      },
      {
        token: 'string',
        regex: String.raw`^\"(?:[^\"\\]|\\[\s\S])*(?:\"|$)`,
        beginWord: false
      },
      {
        token: 'keyword',
        regex: String.raw`(namespace|open|as|operation|function|body|adjoint|newtype|controlled|internal)\b`,
        beginWord: true
      },
      {
        token: 'keyword',
        regex: String.raw`(if|elif|else|repeat|until|fixup|for|in|return|fail|within|apply)\b`,
        beginWord: true
      },
      {
        token: 'keyword',
        regex: String.raw`(Adjoint|Controlled|Adj|Ctl|is|self|auto|distribute|invert|intrinsic)\b`,
        beginWord: true
      },
      {
        token: 'keyword',
        regex: String.raw`(let|set|w\/|new|not|and|or|use|borrow|using|borrowing|newtype|mutable)\b`,
        beginWord: true
      },
      {
        token: 'meta',
        regex: String.raw`(Int|BigInt|Double|Bool|Qubit|Pauli|Result|Range|String|Unit)\b`,
        beginWord: true
      },
      {
        token: 'atom',
        regex: String.raw`(true|false|Pauli(I|X|Y|Z)|One|Zero)\b`,
        beginWord: true
      },
      {
        token: 'builtin',
        regex: String.raw`(X|Y|Z|H|HY|S|T|SWAP|CNOT|CCNOT|MultiX|R|RFrac|Rx|Ry|Rz|R1|R1Frac|Exp|ExpFrac|Measure|M|MultiM)\b`,
        beginWord: true
      },
      {
        token: 'builtin',
        regex: String.raw`(Message|Length|Assert|AssertProb|AssertEqual)\b`,
        beginWord: true
      }
    ];
    let simpleRules = [];
    for (let rule of rules) {
      simpleRules.push({
        token: rule.token,
        regex: new RegExp(rule.regex, 'g'),
        sol: rule.beginWord
      });
      if (rule.beginWord) {
        // Need an additional rule due to the fact that CodeMirror simple mode doesn't work with ^ token
        simpleRules.push({
          token: rule.token,
          regex: new RegExp(String.raw`\W` + rule.regex, 'g'),
          sol: false
        });
      }
    }

    // Register the mode defined above with CodeMirror
    codemirror.CodeMirror.defineSimpleMode('qsharp', { start: simpleRules });
    codemirror.CodeMirror.defineMIME('text/x-qsharp', 'qsharp');

    console.log(codemirror.CodeMirror.modes);
    console.log(codemirror.CodeMirror.mimeModes);

    notebookTracker.currentChanged.connect((sender, args) => {
      console.log('current notebook changed.');
      console.log(sender);
      console.log(args);

      if (args) {
        args.content.modelContentChanged.connect((sender, args) => {
          console.log('notebook model content changed.');
          console.log(sender);

          sender.widgets.forEach(c => {
            if (c.model.type === 'code') {
              c.editor.model.mimeTypeChanged.connect((sender, args) => {
                console.trace();
                console.log(
                  `mime type changed! ${args.name} ${args.oldValue} ${args.newValue}`
                );
                sender.value
              });

              const cellMagic =
                (c.model.value.text.startsWith('%%qsharp') && 'qsharp') ||
                (c.model.value.text.startsWith('%%javascript') && 'javascript');

              if (cellMagic) {
                if (cellMagic === 'qsharp') {
                  console.log("updating type to 'text/x-qsharp'");
                  c.model.mimeType = 'text/x-qsharp';
                } else if (cellMagic === 'javascript') {
                  console.log("updating type to 'text/javascript'");
                  c.model.mimeType = 'text/javascript';
                }
                
                console.log(c.editor);
                console.log(c.editor.model.mimeType);
                console.log(c.editor.getOption('mode' as any));
                c.editor.setOption('mode' as any, cellMagic);
              }
            }
          });
        });
      }
    });
  }
};

export default plugin;
