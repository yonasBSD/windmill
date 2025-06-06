import { initServices } from 'monaco-languageclient/vscode/services'
// import getThemeServiceOverride from '@codingame/monaco-vscode-theme-service-override'
// import getTextmateServiceOverride from '@codingame/monaco-vscode-textmate-service-override'
import getMonarchServiceOverride from '@codingame/monaco-vscode-monarch-service-override'
import '@codingame/monaco-vscode-standalone-typescript-language-features'
import getConfigurationServiceOverride from '@codingame/monaco-vscode-configuration-service-override'
import { editor as meditor, Uri as mUri } from 'monaco-editor'

export let isInitialized = false
export let isInitializing = false

import { getEnhancedMonacoEnvironment } from 'monaco-languageclient/vscode/services'

export function buildWorkerDefinition() {
	const envEnhanced = getEnhancedMonacoEnvironment()

	const getWorker = (moduleId: string, label: string) => {
		console.log(`getWorker: moduleId: ${moduleId} label: ${label}`)

		let selector = label

		// const defaultTextEditorWorker = () => new Worker(
		// 	new URL('@codingame/monaco-vscode-editor-api/esm/vs/editor/editor.worker.js', import.meta.url),
		// 	{ type: 'module' }
		// );
		// const defaultTextMateWorker = () => new Worker(
		// 	new URL('@codingame/monaco-vscode-textmate-service-override/worker', import.meta.url),
		// 	{ type: 'module' }
		// );

		let workerLoaders = {
			TextEditorWorker: () => {
				return new Worker(
					new URL(
						'@codingame/monaco-vscode-editor-api/esm/vs/editor/editor.worker.js',
						import.meta.url
					),
					{
						type: 'module'
					}
				)
			},
			// javascript: () => {
			// 	return new Worker(new URL('monaco-editor-wrapper/workers/module/ts', import.meta.url), {
			// 		type: 'module'
			// 	})
			// },
			javascript: () => {
				return new Worker(
					new URL(
						'@codingame/monaco-vscode-standalone-typescript-language-features/worker',
						import.meta.url
					),
					{
						type: 'module'
					}
				)
			},
			typescript: () => {
				return new Worker(
					new URL(
						'@codingame/monaco-vscode-standalone-typescript-language-features/worker',
						import.meta.url
					),
					{
						type: 'module'
					}
				)
			},
			json: () => {
				return new Worker(
					new URL(
						'@codingame/monaco-vscode-standalone-json-language-features/worker',
						import.meta.url
					),
					{
						type: 'module'
					}
				)
			},
			html: () => {
				return new Worker(
					new URL(
						'@codingame/monaco-vscode-standalone-html-language-features/worker',
						import.meta.url
					),
					{
						type: 'module'
					}
				)
			},
			css: () => {
				return new Worker(
					new URL(
						'@codingame/monaco-vscode-standalone-css-language-features/worker',
						import.meta.url
					),
					{
						type: 'module'
					}
				)
			},
			graphql: () => {
				console.log('Creating graphql worker')
				return new Worker(new URL(`../monaco_workers/graphql.worker.bundle.js`, import.meta.url), {
					name: 'graphql'
				})
			}
		}
		const workerFunc = workerLoaders[selector]
		if (workerFunc !== undefined) {
			return workerFunc()
		} else {
			throw new Error(`Unimplemented worker ${label} (${moduleId})`)
		}
	}
	envEnhanced.getWorker = getWorker
}

export async function initializeVscode(caller?: string, htmlContainer?: HTMLElement) {
	if (!isInitialized && !isInitializing) {
		console.log(`Initializing vscode-api from ${caller ?? 'unknown'}`)
		isInitializing = true

		try {
			// init vscode-api
			await initServices(
				{
					serviceOverrides: {
						// ...getThemeServiceOverride(),
						// ...getTextmateServiceOverride()
						...getConfigurationServiceOverride(),
						...getMonarchServiceOverride()
					},
					enableExtHostWorker: false,
					userConfiguration: {
						json: JSON.stringify({
							'editor.experimental.asyncTokenization': true
						})
					}
				},
				{
					monacoWorkerFactory: buildWorkerDefinition
				}
			)

			isInitialized = true
			meditor.defineTheme('nord', {
				base: 'vs-dark',
				inherit: true,
				rules: [
					{
						background: '2E3440',
						token: ''
					},
					{
						foreground: '808b9f',
						token: 'comment'
					},
					{
						foreground: 'a3be8c',
						token: 'string'
					},
					{
						foreground: 'b48ead',
						token: 'constant.numeric'
					},
					{
						foreground: '81a1c1',
						token: 'constant.language'
					},
					{
						foreground: '81a1c1',
						token: 'keyword'
					},
					{
						foreground: '81a1c1',
						token: 'storage'
					},
					{
						foreground: '81a1c1',
						token: 'storage.type'
					},
					{
						foreground: '8fbcbb',
						token: 'entity.name.class'
					},
					{
						foreground: '8fbcbb',
						fontStyle: '  bold',
						token: 'entity.other.inherited-class'
					},
					{
						foreground: '88c0d0',
						token: 'entity.name.function'
					},
					{
						foreground: '81a1c1',
						token: 'entity.name.tag'
					},
					{
						foreground: '8fbcbb',
						token: 'entity.other.attribute-name'
					},
					{
						foreground: '88c0d0',
						token: 'support.function'
					},
					{
						foreground: 'f8f8f0',
						background: 'f92672',
						token: 'invalid'
					},
					{
						foreground: 'f8f8f0',
						background: 'ae81ff',
						token: 'invalid.deprecated'
					},
					{
						foreground: 'b48ead',
						token: 'constant.color.other.rgb-value'
					},
					{
						foreground: 'ebcb8b',
						token: 'constant.character.escape'
					},
					{
						foreground: '8fbcbb',
						token: 'variable.other.constant'
					}
				],
				colors: {
					'editor.foreground': '#D8DEE9',
					'editor.background': '#272D38',
					'editor.selectionBackground': '#434C5ECC',
					'editor.lineHighlightBackground': '#3B4252',
					'editorCursor.foreground': '#D8DEE9',
					'editorWhitespace.foreground': '#434C5ECC'
				}
			})

			meditor.defineTheme('myTheme', {
				base: 'vs',
				inherit: true,
				rules: [],
				colors: {
					'editorLineNumber.foreground': '#999',
					'editorGutter.background': '#F9FAFB'
				}
			})

			if (document.documentElement.classList.contains('dark')) {
				meditor.setTheme('nord')
			} else {
				meditor.setTheme('myTheme')
			}
		} catch (e) {
			console.error('Failed to initialize monaco services', e)
		} finally {
			isInitialized = true
			isInitializing = false
		}
	} else {
		while (isInitializing && !isInitialized) {
			console.log('Waiting for initialization of monaco services')
			await new Promise((resolve) => setTimeout(resolve, 100))
		}
	}
}

export function keepModelAroundToAvoidDisposalOfWorkers() {
	const keepEditorUri = mUri.parse('file:///avoidDisposalOfWorkers')
	if (!meditor?.getModel(keepEditorUri)) {
		meditor.createModel('', 'typescript', keepEditorUri)
	}
}
