// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


import * as vscode from 'vscode';
import { loadProject } from './projectSystem';
import { getCompilerWorker, ICompilerWorker, IProjectConfig, log, QscEventTarget } from "qsharp-lang";
import { getActiveQSharpDocumentUri } from './programConfig';
import { IProgramConfig } from '../../npm/qsharp/lib/web/qsc_wasm';
import { getTarget } from './config';
import { toVsCodeRange } from './common';


// TODO(sezna):
// - handle running all tests
// - Auto-populate newly discovered tests
// - CodeLens
// - Cancellation tokens
function localGetCompilerWorker(context: vscode.ExtensionContext): ICompilerWorker {
	const compilerWorkerScriptPath = vscode.Uri.joinPath(
		context.extensionUri,
		"./out/compilerWorker.js",
	).toString();
	const worker = getCompilerWorker(compilerWorkerScriptPath);
	return worker;
}

async function getProgramConfig(): Promise<IProgramConfig | null> {
	if (!vscode.workspace.workspaceFolders) {
		log.info("No workspace detected; not starting test explorer")
		return null;
	}

	const docUri = getActiveQSharpDocumentUri();
	if (!docUri) {
		log.info("No active document detected; not starting test explorer")
		return null;
	}

	const projectConfig: IProjectConfig = await loadProject(docUri);
	if (!projectConfig) {
		log.info("No project detected; not starting test explorer")
		return null;
	}

	return {
		profile: getTarget(),
		...projectConfig
	}
}

/** 
 * Constructs the handler to pass to the `TestController` that refreshes the discovered tests.
 * if `shouldDeleteOldTests` is `true`, then clear out previously discovered tests. If `false`, add new tests but don't dissolve old ones.
 * 
 */
function mkRefreshHandler(ctrl: vscode.TestController, context: vscode.ExtensionContext, shouldDeleteOldTests: boolean = true) {
	return async () => {
		if (shouldDeleteOldTests) {
			for (const [id, _] of ctrl.items) {
				ctrl.items.delete(id);
			}
		}
		const programConfig = await getProgramConfig();
		if (!programConfig) {
			return;
		}
		const worker = localGetCompilerWorker(context);

		const testCallables = await worker.collectTestCallables(programConfig);

		// break down the test callable into its parts, so we can construct 
		// the namespace hierarchy in the test explorer
		const hierarchy: {[key: string]: vscode.TestItem} = {};

		for (const testCallable of testCallables) {
			const parts = testCallable.split('.');
			
			// for an individual test case, e.g. foo.bar.baz, create a hierarchy of items
			let rover = ctrl.items;
			for (let i = 0; i < parts.length; i++) {
				const part = parts[i];
				const id = i === parts.length - 1 ? testCallable : part;
				if (!rover.get(part)) {
					rover.add(ctrl.createTestItem(id, part));
				}
				rover = rover.get(id)!.children;
			}
		}
	}
}


const fileChangedEmitter = new vscode.EventEmitter<vscode.Uri>();


export async function initTestExplorer(context: vscode.ExtensionContext) {
	const ctrl: vscode.TestController = vscode.tests.createTestController('qsharpTestController', 'Q# Tests');
	context.subscriptions.push(ctrl);
	context.subscriptions.push(
		vscode.commands.registerCommand(
		  "qsharp-vscode.runTest",
		  // TODO: codelens callback
		  () => {},
		)
	  )

	// construct the handler that runs when the user presses the refresh button in the test explorer
	const refreshHandler = mkRefreshHandler(ctrl, context);
	// initially populate tests
	await refreshHandler();

	ctrl.refreshHandler = refreshHandler;

	const runHandler = (request: vscode.TestRunRequest, cancellation: vscode.CancellationToken) => {
		if (!request.continuous) {
			return startTestRun(request);
		}
	};

	const startTestRun = async (request: vscode.TestRunRequest) => {
		// use the compiler worker to run the test in the interpreter

		log.info("Starting test run, request was", JSON.stringify(request));
		const worker = localGetCompilerWorker(context);

		let program = await getProgramConfig();
		if (!program) {
			return;
		}

		const queue = [];

		for (const testCase of request.include || []) {
			const run = ctrl.createTestRun(request);
			const testRunFunc = async () => {
				const evtTarget = new QscEventTarget(false);
				evtTarget.addEventListener('Message', (msg) => {
					run.appendOutput(`Test ${testCase.id}: ${msg.detail}\r\n`);

				})

				evtTarget.addEventListener('Result', (msg) => {
					if (msg.detail.success) {
						run.passed(testCase);
					} else {
						let message: vscode.TestMessage = {
							message: msg.detail.value.message,
							location: {
								range: toVsCodeRange(msg.detail.value.range),
								uri: vscode.Uri.parse(msg.detail.value.uri || "")
							}
						}
						run.failed(testCase, message);
					}
					run.end();
				})
				const callableExpr = `${testCase.id}()`;
				log.info("about to run test", callableExpr);
				try {
					await worker.run(program, callableExpr, 1, evtTarget);
				} catch (error) {
					log.error(`Error running test ${testCase.id}:`, error);
					run.appendOutput(`Error running test ${testCase.id}: ${error}\r\n`);
				}
				log.info("ran test", testCase.id);

			}

			queue.push(testRunFunc);
		}

		for (const func of queue) {
			await func();
		}



		/*
		example: 
		{
			"include":[
				{
					"id":"Main",
					"children":[],
					"label":"Main",
					"canResolveChildren":false,
					"busy":false,
					"tags":[]
				}
			],
			"exclude":[],
			"profile":{
				"controllerId":"qsharpTestController",
				"profileId":1933983363,
				"kind":1,
				"g":"Run Tests",
				"j":true
			},
			"continuous":false,
			"preserveFocus":true}
		*/

		// map of file uris to statements on each line:
		// const coveredLines = new Map</* file uri */ string, (vscode.StatementCoverage | undefined)[]>();
		// run the test TODO
	};

	ctrl.createRunProfile('Run Tests', vscode.TestRunProfileKind.Run, runHandler, true, undefined, true);

	ctrl.resolveHandler = async item => {
		if (!item) {
			context.subscriptions.push(...startWatchingWorkspace(ctrl, fileChangedEmitter));
			return;
		}

	};

	function updateNodeForDocument(e: vscode.TextDocument) {
		if (e.uri.scheme !== 'file') {
			return;
		}

		if (!e.uri.path.endsWith('.qs')) {
			return;
		}

		// const { file, data } = getOrCreateFile(ctrl, e.uri);
		// data.updateFromContents(ctrl, e.getText(), file);
	}

	for (const document of vscode.workspace.textDocuments) {
		updateNodeForDocument(document);
	}

	context.subscriptions.push(
		vscode.workspace.onDidOpenTextDocument(updateNodeForDocument),
		vscode.workspace.onDidChangeTextDocument(e => updateNodeForDocument(e.document)),
	);
}



function gatherTestItems(collection: vscode.TestItemCollection) {
	const items: vscode.TestItem[] = [];
	collection.forEach(item => items.push(item));
	return items;
}

function getWorkspaceTestPatterns() {
	if (!vscode.workspace.workspaceFolders) {
		return [];
	}

	return vscode.workspace.workspaceFolders.map(workspaceFolder => ({
		workspaceFolder,
		pattern: new vscode.RelativePattern(workspaceFolder, '**/*.qs'),
	}));
}


function startWatchingWorkspace(controller: vscode.TestController, fileChangedEmitter: vscode.EventEmitter<vscode.Uri>) {
	return getWorkspaceTestPatterns().map(({ pattern }) => {
		const watcher = vscode.workspace.createFileSystemWatcher(pattern);
		/*
				watcher.onDidCreate(uri => {
					getOrCreateFile(controller, uri);
					fileChangedEmitter.fire(uri);
				});
				watcher.onDidChange(async uri => {
					const { file, data } = getOrCreateFile(controller, uri);
					if (data.didResolve) {
						await data.updateFromDisk(controller, file);
					}
					fileChangedEmitter.fire(uri);
				});
				*/
		watcher.onDidDelete(uri => controller.items.delete(uri.toString()));

		// findInitialFiles(controller, pattern);

		return watcher;
	});
}