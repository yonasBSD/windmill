<script lang="ts">
	import LogViewer from './LogViewer.svelte'
	import JobProgressBar from '$lib/components/jobs/JobProgressBar.svelte'
	import ScriptFix from './copilot/ScriptFix.svelte'
	import type DiffEditor from './DiffEditor.svelte'
	import type Editor from './Editor.svelte'
	import type { Script, Job, FlowModule } from '$lib/gen'
	import OutputPickerInner from '$lib/components/flows/propPicker/OutputPickerInner.svelte'
	import { Pane, Splitpanes } from 'svelte-splitpanes'
	import type { FlowEditorContext } from './flows/types'
	import { getContext } from 'svelte'
	import { getStringError } from './copilot/chat/utils'

	interface Props {
		lang: Script['language']
		editor: Editor | undefined
		diffEditor: DiffEditor | undefined
		loopStatus?: { type: 'inside' | 'self'; flow: 'forloopflow' | 'whileloopflow' } | undefined
		lastJob?: Job | undefined
		scriptProgress?: number | undefined
		testJob?: Job | undefined
		mod: FlowModule
		testIsLoading?: boolean
		disableMock?: boolean
		disableHistory?: boolean
		onUpdateMock?: (mock: { enabled: boolean; return_value?: unknown }) => void
		loadingJob?: boolean
		tagLabel?: string
	}

	let {
		lang,
		editor,
		diffEditor,
		loopStatus = undefined,
		lastJob = undefined,
		scriptProgress = $bindable(undefined),
		testJob = undefined,
		mod,
		testIsLoading = false,
		disableMock = false,
		disableHistory = false,
		onUpdateMock,
		loadingJob = false,
		tagLabel = undefined
	}: Props = $props()

	const { testSteps } = getContext<FlowEditorContext>('FlowEditorContext')

	let selectedJob: Job | undefined = $state(undefined)
	let preview: 'mock' | 'job' | undefined = $state(undefined)
	let jobProgressReset: () => void = $state(() => {})

	let forceJson = $state(false)

	const logJob = $derived(testJob ?? selectedJob)
</script>

<Splitpanes horizontal>
	<Pane size={65} minSize={10} class="text-sm text-tertiary">
		{#if scriptProgress}
			<JobProgressBar
				job={testJob}
				bind:scriptProgress
				bind:reset={jobProgressReset}
				compact={true}
			/>
		{/if}

		<OutputPickerInner
			{lastJob}
			{testJob}
			fullResult
			moduleId={mod.id}
			closeOnOutsideClick={true}
			getLogs
			{onUpdateMock}
			mock={mod.mock}
			bind:forceJson
			bind:selectedJob
			isLoading={testIsLoading || loadingJob}
			bind:preview
			path={`path` in mod.value ? mod.value.path : ''}
			{loopStatus}
			{disableMock}
			{disableHistory}
		>
			{#snippet copilot_fix()}
				{#if lang && editor && diffEditor && testSteps.getStepArgs(mod.id) && selectedJob?.type === 'CompletedJob' && !selectedJob.success && getStringError(selectedJob.result)}
					<ScriptFix {lang} />
				{/if}
			{/snippet}
		</OutputPickerInner>
	</Pane>
	<Pane size={35} minSize={10}>
		{#if (mod.mock?.enabled && preview != 'job') || preview == 'mock'}
			<LogViewer
				small
				content={undefined}
				isLoading={false}
				tag={undefined}
				customEmptyMessage="Using pinned data"
				{tagLabel}
			/>
		{:else}
			<LogViewer
				small
				jobId={logJob?.id}
				duration={logJob?.['duration_ms']}
				mem={logJob?.['mem_peak']}
				content={logJob?.logs}
				isLoading={(testIsLoading && logJob?.['running'] == false) || loadingJob}
				tag={logJob?.tag}
				{tagLabel}
			/>
		{/if}
	</Pane>
</Splitpanes>
