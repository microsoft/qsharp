import {
  JupyterFrontEnd,
  JupyterFrontEndPlugin
} from '@jupyterlab/application';

import * as codemirror from '@jupyterlab/codemirror';
import * as simpleMode from '@codemirror/legacy-modes/mode/simple-mode';
import * as notebook from '@jupyterlab/notebook';
import { LanguageSupport, StreamLanguage } from '@codemirror/language';

/**
 * Initialization data for the qsharp_jupyterlab extension.
 */
const plugin: JupyterFrontEndPlugin<void> = {
  id: 'qsharp',
  autoStart: true,
  requires: [codemirror.IEditorLanguageRegistry, notebook.INotebookTracker],
  activate: async (
    app: JupyterFrontEnd,
    codemirrorLanguageRegistry: codemirror.IEditorLanguageRegistry,
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

    const parser = simpleMode.simpleMode({ start: simpleRules });

    const languageSupport = new LanguageSupport(StreamLanguage.define(parser));

    codemirrorLanguageRegistry.addLanguage({
      name: 'qsharp',
      mime: 'text/x-qsharp',
      support: languageSupport
    });

    notebookTracker.currentChanged.connect((notebookTracker, notebookPanel) => {
      console.log('current notebook changed.');

      if (notebookPanel) {
        // TODO: I'm pretty sure I'm attaching too many handlers
        notebookPanel.content.modelContentChanged.connect((sender, args) => {
          console.log('notebook model content changed.');

          for (const cell of sender.widgets) {
            if (cell.model.type === 'code') {
              cell.ready.then(() => {
                if (
                  cell.model.sharedModel.source.startsWith('%%qsharp') &&
                  cell.model.mimeType !== 'text/x-qsharp'
                ) {
                  console.log("updating type to 'text/x-qsharp'");
                  cell.model.mimeType = 'text/x-qsharp';
                  console.log(cell.editorConfig);
                  cell.update();
                }
              });
            }
          }
        });
      }
    });
  }
};

export default plugin;
