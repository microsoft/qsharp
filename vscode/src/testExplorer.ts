// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


import * as vscode from 'vscode';
import { loadProject } from './projectSystem';
import { IProjectConfig, log } from "qsharp-lang";
import { getActiveQSharpDocumentUri } from './programConfig';

export async function initTestExplorer(context: vscode.ExtensionContext) {
	const ctrl: vscode.TestController = vscode.tests.createTestController('qsharpTestController', 'Q# Tests');
	context.subscriptions.push(ctrl);
	const item = ctrl.createTestItem("Q# test","test fn");
	ctrl.items.add(item);


	ctrl.refreshHandler = async () => {
		log.info("1");
		if (!vscode.workspace.workspaceFolders) {
			log.info("No workspace detected; not starting test explorer")
			return;
		} 

		log.info("2");
		const docUri =  getActiveQSharpDocumentUri();
		if (!docUri) {
			log.info("No active document detected; not starting test explorer")
			return;
		}


		const projectConfig: IProjectConfig = await loadProject(docUri);
		if (!projectConfig) {
			log.info("No project detected; not starting test explorer")
			return;
		}
		log.info("3");

		const sources = projectConfig.packageGraphSources.root.sources;
		for (const [sourceUrl, sourceContent] of sources) {
			const testItem = ctrl.createTestItem(sourceUrl, sourceUrl);
			ctrl.items.add(testItem);
		}
		log.info("4");

		
	};

	const fileChangedEmitter = new vscode.EventEmitter<vscode.Uri>();
	const watchingTests = new Map<vscode.TestItem | 'ALL', vscode.TestRunProfile | undefined>();
	fileChangedEmitter.event(uri => {
		if (watchingTests.has('ALL')) {
			startTestRun(new vscode.TestRunRequest(undefined, undefined, watchingTests.get('ALL'), true));
			return;
		}

		const include: vscode.TestItem[] = [];
		let profile: vscode.TestRunProfile | undefined;
		for (const [item, thisProfile] of watchingTests) {
			const cast = item as vscode.TestItem;
			if (cast.uri?.toString() == uri.toString()) {
				include.push(cast);
				profile = thisProfile;
			}
		}

		if (include.length) {
			startTestRun(new vscode.TestRunRequest(include, undefined, profile, true));
		}
	});

	const runHandler = (request: vscode.TestRunRequest, cancellation: vscode.CancellationToken) => {
		if (!request.continuous) {
			return startTestRun(request);
		}

		if (request.include === undefined) {
			watchingTests.set('ALL', request.profile);
			cancellation.onCancellationRequested(() => watchingTests.delete('ALL'));
		} else {
			request.include.forEach(item => watchingTests.set(item, request.profile));
			cancellation.onCancellationRequested(() => request.include!.forEach(item => watchingTests.delete(item)));
		}
	};

	const startTestRun = (request: vscode.TestRunRequest) => {
		const queue: { test: vscode.TestItem; data: string }[] = [];
		const run = ctrl.createTestRun(request);
		// map of file uris to statements on each line:
		// const coveredLines = new Map</* file uri */ string, (vscode.StatementCoverage | undefined)[]>();
		// run the test TODO
		vscode.window.showInformationMessage('Running tests...');
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