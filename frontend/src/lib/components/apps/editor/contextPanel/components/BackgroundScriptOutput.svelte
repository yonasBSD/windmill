<script lang="ts">
	import type { AppViewerContext } from '$lib/components/apps/types'
	import { getContext } from 'svelte'
	import { connectInput } from '../../appUtils'
	import ComponentOutputViewer from '../ComponentOutputViewer.svelte'
	import OutputHeader from './OutputHeader.svelte'

	const { connectingInput } = getContext<AppViewerContext>('AppViewerContext')

	export let id: string
	export let name: string
	export let first: boolean = false
</script>

<OutputHeader let:render renamable={false} selectable={true} {id} {name} color="blue" {first}>
	<ComponentOutputViewer
		{render}
		componentId={id}
		on:select={({ detail }) => {
			$connectingInput = connectInput($connectingInput, id, detail)
		}}
	/>
</OutputHeader>
