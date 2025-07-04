<script lang="ts">
	import type { Schema } from '$lib/common'
	import { VariableService, type InputTransform } from '$lib/gen'
	import { workspaceStore } from '$lib/stores'
	import { allTrue } from '$lib/utils'
	import { untrack } from 'svelte'
	import { Button } from './common'
	import StepInputsGen from './copilot/StepInputsGen.svelte'
	import type { PickableProperties } from './flows/previousResults'
	import InputTransformForm from './InputTransformForm.svelte'
	import ItemPicker from './ItemPicker.svelte'
	import VariableEditor from './VariableEditor.svelte'
	import { Plus } from 'lucide-svelte'

	interface Props {
		schema: Schema | { properties?: Record<string, any> }
		args?: Record<string, InputTransform | any>
		isValid?: boolean
		extraLib?: string
		previousModuleId?: string | undefined
		filter?: string[] | undefined
		noDynamicToggle?: boolean
		pickableProperties?: PickableProperties | undefined
		enableAi?: boolean
		class?: string
	}

	let {
		schema = $bindable(),
		args = $bindable({}),
		isValid = $bindable(true),
		extraLib = $bindable('missing extraLib'),
		previousModuleId = undefined,
		filter = undefined,
		noDynamicToggle = false,
		pickableProperties = undefined,
		enableAi = false,
		class: clazz = ''
	}: Props = $props()

	let inputCheck: { [id: string]: boolean } = $state({})

	$effect(() => {
		isValid = allTrue(inputCheck) ?? false
	})

	$effect(() => {
		if (args == undefined || typeof args !== 'object') {
			args = {}
		}
	})

	export function setArgs(nargs: Record<string, InputTransform | any>) {
		args = nargs
	}

	function removeExtraKey() {
		const nargs = {}
		Object.keys(args ?? {}).forEach((key) => {
			if (keys.includes(key)) {
				nargs[key] = args[key]
			}
		})
		args = nargs
	}

	let pickForField: string | undefined = $state()
	let itemPicker: ItemPicker | undefined = $state(undefined)
	let variableEditor: VariableEditor | undefined = $state(undefined)

	let keys: string[] = $state([])
	$effect(() => {
		let lkeys = Object.keys(schema?.properties ?? {})
		if (schema?.properties && JSON.stringify(lkeys) != JSON.stringify(keys)) {
			keys = lkeys
			untrack(() => removeExtraKey())
		}
	})
</script>

<div class="w-full {clazz}">
	{#if enableAi}
		<div class="px-0.5 pt-0.5">
			<StepInputsGen
				{pickableProperties}
				argNames={keys
					? keys.filter(
							(argName) =>
								Object.keys(schema.properties ?? {}).includes(argName) &&
								Object.keys(args ?? {}).includes(argName) &&
								((args[argName].type === 'static' && !args[argName].value) ||
									(args[argName].type === 'javascript' && !args[argName].expr))
						)
					: []}
				{schema}
			/>
		</div>
	{/if}
	{#if keys.length > 0}
		{#each keys as argName, index (argName)}
			{#if (!filter || filter.includes(argName)) && Object.keys(schema.properties ?? {}).includes(argName)}
				<div class="pt-2 relative">
					<InputTransformForm
						{previousModuleId}
						bind:arg={args[argName]}
						bind:schema
						bind:argName={keys[index]}
						bind:inputCheck={
							() => inputCheck[argName] ?? false, (value) => (inputCheck[argName] = value)
						}
						bind:extraLib
						{variableEditor}
						{itemPicker}
						bind:pickForField
						{noDynamicToggle}
						{pickableProperties}
						{enableAi}
					/>
				</div>
			{/if}
		{/each}
	{:else}
		<div class="text-tertiary text-sm">No inputs</div>
	{/if}
</div>

<ItemPicker
	bind:this={itemPicker}
	pickCallback={(path, _) => {
		if (pickForField) {
			args[pickForField].value = '$var:' + path
		}
	}}
	itemName="Variable"
	extraField="path"
	loadItems={async () =>
		(await VariableService.listVariable({ workspace: $workspaceStore ?? '' })).map((x) => ({
			name: x.path,
			...x
		}))}
>
	{#snippet submission()}
		<div class="flex flex-row-reverse w-full border-t border-gray-200 rounded-bl-lg rounded-br-lg">
			<Button
				variant="border"
				color="blue"
				size="sm"
				startIcon={{ icon: Plus }}
				on:click={() => {
					variableEditor?.initNew?.()
				}}
			>
				New variable
			</Button>
		</div>
	{/snippet}
</ItemPicker>

<VariableEditor bind:this={variableEditor} />
