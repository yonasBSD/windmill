<script lang="ts">
	import { preventDefault, stopPropagation, createBubbler } from 'svelte/legacy'

	const bubble = createBubbler()
	import type { EnumType, SchemaProperty } from '$lib/common'
	import {
		setInputCat as computeInputCat,
		debounce,
		emptyString,
		getSchemaFromProperties
	} from '$lib/utils'
	import { DollarSign, Plus, X, Check, Loader2, ExternalLink } from 'lucide-svelte'
	import { createEventDispatcher, onDestroy, onMount, tick, untrack } from 'svelte'
	import { fade } from 'svelte/transition'
	import { Button, SecondsInput } from './common'
	import FieldHeader from './FieldHeader.svelte'
	import type ItemPicker from './ItemPicker.svelte'
	import ObjectResourceInput from './ObjectResourceInput.svelte'
	import Range from './Range.svelte'
	import ResourcePicker from './ResourcePicker.svelte'
	import type SimpleEditor from './SimpleEditor.svelte'
	import Toggle from './Toggle.svelte'
	import type VariableEditor from './VariableEditor.svelte'
	import { twMerge } from 'tailwind-merge'
	import ArgEnum from './ArgEnum.svelte'
	import DateTimeInput from './DateTimeInput.svelte'
	import DateInput from './DateInput.svelte'
	import CurrencyInput from './apps/components/inputs/currency/CurrencyInput.svelte'
	import autosize from '$lib/autosize'
	import PasswordArgInput from './PasswordArgInput.svelte'
	import Password from './Password.svelte'
	import ToggleButtonGroup from './common/toggleButton-v2/ToggleButtonGroup.svelte'
	import ToggleButton from './common/toggleButton-v2/ToggleButton.svelte'
	import SchemaFormDnd from './schema/SchemaFormDND.svelte'
	import SchemaForm from './SchemaForm.svelte'
	import { deepEqual } from 'fast-equals'
	import DynSelect from './DynSelect.svelte'
	import type { Script } from '$lib/gen'
	import type { SchemaDiff } from '$lib/components/schema/schemaUtils.svelte'
	import type { ComponentCustomCSS } from './apps/types'
	import MultiSelect from './select/MultiSelect.svelte'
	import { safeSelectItems } from './select/utils.svelte'
	import S3ArgInput from './common/fileUpload/S3ArgInput.svelte'
	import { base } from '$lib/base'

	interface Props {
		label?: string
		value: any
		defaultValue?: any
		description?: string
		format?: string
		contentEncoding?: 'base64' | 'binary' | undefined
		type?: string | undefined
		oneOf?: SchemaProperty[] | undefined
		required?: boolean
		pattern?: undefined | string
		valid?: boolean
		enum_?: EnumType
		disabled?: boolean
		itemsType?:
			| {
					type?: 'string' | 'number' | 'bytes' | 'object' | 'resource'
					contentEncoding?: 'base64'
					enum?: string[]
					multiselect?: string[]
					resourceType?: string
					properties?: { [name: string]: SchemaProperty }
			  }
			| undefined
		displayHeader?: boolean
		properties?: { [name: string]: SchemaProperty } | undefined
		nestedRequired?: string[] | undefined
		autofocus?: boolean | null
		compact?: boolean
		password?: boolean
		pickForField?: string | undefined
		variableEditor?: VariableEditor | undefined
		itemPicker?: ItemPicker | undefined
		noMargin?: boolean
		extra?: Record<string, any>
		minW?: boolean
		prettifyHeader?: boolean
		resourceTypes: string[] | undefined
		disablePortal?: boolean
		showSchemaExplorer?: boolean
		simpleTooltip?: string | undefined
		customErrorMessage?: string | undefined
		onlyMaskPassword?: boolean
		nullable?: boolean
		title?: string | undefined
		placeholder?: string | undefined
		order?: string[] | undefined
		editor?: SimpleEditor | undefined
		orderEditable?: boolean
		shouldDispatchChanges?: boolean
		noDefaultOnSelectFirst?: boolean
		helperScript?:
			| { type: 'inline'; path?: string; lang: Script['language']; code: string }
			| { type: 'hash'; hash: string }
			| undefined
		otherArgs?: Record<string, any>
		lightHeader?: boolean
		diffStatus?: SchemaDiff | undefined
		hideNested?: boolean
		nestedParent?: { label: string; nestedParent: any | undefined } | undefined
		nestedClasses?: string
		displayType?: boolean
		css?: ComponentCustomCSS<'schemaformcomponent'> | undefined
		appPath?: string | undefined
		computeS3ForceViewerPolicies?:
			| (() =>
					| {
							allowed_resources: string[]
							allow_user_resources: boolean
							allow_workspace_resource: boolean
							file_key_regex: string
					  }
					| undefined)
			| undefined
		workspace?: string | undefined
		actions?: import('svelte').Snippet
	}

	let {
		label = '',
		value = $bindable(),
		defaultValue = $bindable(undefined),
		description = $bindable(undefined),
		format = $bindable(undefined),
		contentEncoding = undefined,
		type = undefined,
		oneOf = $bindable(undefined),
		required = false,
		pattern = $bindable(undefined),
		valid = $bindable(undefined),
		enum_ = $bindable(undefined),
		disabled = false,
		itemsType = $bindable(undefined),
		displayHeader = true,
		properties = $bindable(undefined),
		nestedRequired = undefined,
		autofocus = null,
		compact = false,
		password = false,
		pickForField = $bindable(undefined),
		variableEditor = undefined,
		itemPicker = undefined,
		noMargin = false,
		extra = {},
		minW = true,
		prettifyHeader = false,
		resourceTypes,
		disablePortal = false,
		showSchemaExplorer = false,
		simpleTooltip = undefined,
		customErrorMessage = undefined,
		onlyMaskPassword = false,
		nullable = false,
		title = $bindable(undefined),
		placeholder = $bindable(undefined),
		order = $bindable(undefined),
		editor = $bindable(undefined),
		orderEditable = false,
		shouldDispatchChanges = false,
		noDefaultOnSelectFirst = false,
		helperScript = undefined,
		otherArgs = {},
		lightHeader = false,
		diffStatus = undefined,
		hideNested = false,
		nestedParent = undefined,
		nestedClasses = '',
		displayType = true,
		css = undefined,
		appPath = undefined,
		computeS3ForceViewerPolicies = undefined,
		workspace = undefined,
		actions
	}: Props = $props()

	$effect(() => {
		if (description == undefined) {
			description = ''
		}
		if (format == undefined) {
			format = ''
		}
		if (valid == undefined) {
			valid = true
		}
	})

	let oneOfSelected: string | undefined = $state(undefined)
	let tagKey = $derived(
		oneOf?.find((o) => Object.keys(o.properties ?? {})?.includes('kind')) ? 'kind' : 'label'
	)
	async function updateOneOfSelected(oneOf: SchemaProperty[] | undefined) {
		if (
			oneOf &&
			oneOf.length >= 2 &&
			(!oneOfSelected || !oneOf.some((o) => o.title === oneOfSelected) || !value)
		) {
			if (value && value[tagKey] && oneOf.some((o) => o.title === value[tagKey])) {
				const existingValue = JSON.parse(JSON.stringify(value))
				oneOfSelected = value[tagKey]
				await tick()
				value = existingValue
			} else {
				const variantTitle = oneOf[0]['title']
				oneOfSelected = variantTitle
				value = { ...(typeof value === 'object' ? (value ?? {}) : {}), [tagKey]: variantTitle }
			}
		}
	}

	function onOneOfChange() {
		const label = value?.['label']
		const kind = value?.['kind']
		if (label && oneOf && oneOf.some((o) => o.title == label) && oneOfSelected != label) {
			oneOfSelected = label
		} else if (kind && oneOf && oneOf.some((o) => o.title == kind) && oneOfSelected != kind) {
			oneOfSelected = kind
		}
	}

	const dispatch = createEventDispatcher()

	let ignoreValueUndefined = $state(false)
	let error: string = $state('')
	let isListJson = $state(false)
	let hasIsListJsonChanged = $state(false)

	let el: HTMLTextAreaElement | undefined = $state(undefined)

	let rawValue: string | undefined = $state(undefined)

	let lastValue: any = null

	function computeDefaultValue(inputCat?: string, defaultValue?: any, nnullable?: boolean) {
		let nvalue: any = null
		if (label == 'toString' && typeof value == 'function') {
			nvalue = undefined
		}
		if ((value == undefined || value == null) && !ignoreValueUndefined) {
			nvalue = structuredClone($state.snapshot(defaultValue))
			if (defaultValue === undefined || defaultValue === null) {
				if (inputCat === 'string') {
					nvalue = nullable ? null : ''
				} else if (inputCat == 'enum' && required) {
					nvalue = enum_?.[0]
				} else if (inputCat == 'boolean') {
					nvalue = false
				} else if (inputCat == 'list') {
					nvalue = []
				}
			} else if (inputCat === 'object') {
				evalValueToRaw()
			}
		}

		if (nnullable && type === 'string' && value === '' && nvalue == null) {
			value = null
		} else if (nvalue != null && !deepEqual(nvalue, value)) {
			value = nvalue
			lastValue = value
		}
	}

	// By setting isListJson to true, we can render inputs even if the value is not an array of the correct type
	// This avoids the issue of the input being rendered as a string with value: [object Object], or as a number with value: NaN
	function checkArrayValueType() {
		try {
			if (Array.isArray(value) && value.length > 0) {
				const firstItem = value?.[0]
				const type = itemsType?.type

				switch (type) {
					case 'string':
						if (typeof firstItem !== 'string') {
							isListJson = true
						}
						break
					case 'number':
						if (typeof firstItem !== 'number') {
							isListJson = true
						}
						break
				}
			}
		} catch (e) {
			console.error(e)
		}

		lastValue = value
	}

	let oldDefaultValue = structuredClone($state.snapshot(defaultValue))
	function handleDefaultValueChange() {
		if (
			deepEqual(value, oldDefaultValue) &&
			!deepEqual(value, defaultValue) &&
			!deepEqual(defaultValue, oldDefaultValue)
		) {
			value = defaultValue
		}
		oldDefaultValue = structuredClone($state.snapshot(defaultValue))
	}

	function isObjectCat(inputCat?: string) {
		return (
			inputCat === 'object' || inputCat === 'resource-object' || (inputCat == 'list' && !isListJson)
		)
	}

	function isRawStringEditor(inputCat?: string) {
		return inputCat == 'sql' || inputCat == 'yaml'
	}

	function evalValueToRaw() {
		if (setCodeDisabled) {
			return
		}
		const newRawValue =
			value == undefined || value == null
				? ''
				: isObjectCat(inputCat) || (inputCat == 'list' && isListJson)
					? JSON.stringify(value, null, 2)
					: isRawStringEditor(inputCat)
						? typeof value == 'string'
							? value
							: JSON.stringify(value, null, 2)
						: undefined

		if (newRawValue != rawValue) {
			rawValue = newRawValue
			rawValue != undefined && editor?.getCode() != rawValue && editor?.setCode(rawValue)
		}
	}

	let setCodeDisabled = false

	let timeout: NodeJS.Timeout | undefined = undefined
	function setNewValueFromCode(nvalue: any) {
		if (!deepEqual(nvalue, value)) {
			value = nvalue
			timeout && clearTimeout(timeout)
			setCodeDisabled = true
			timeout = setTimeout(() => {
				setCodeDisabled = false
			}, 1000)
		}
	}

	onMount(() => {
		evalValueToRaw()
	})

	function fileChanged(e: any, cb: (v: string | undefined) => void) {
		let t = e.target
		if (t && 'files' in t && t.files.length > 0) {
			let reader = new FileReader()
			reader.onload = (e: any) => {
				cb(e.target.result.split('base64,')[1])
			}
			reader.readAsDataURL(t.files[0])
		} else {
			cb(undefined)
		}
	}

	export function focus() {
		el?.focus()
		if (el) {
			el.style.height = '5px'
			el.style.height = el.scrollHeight + 50 + 'px'
		}
	}

	function validateInput(pattern: string | undefined, v: any, required: boolean): void {
		if (nullable && emptyString(v)) {
			error = ''
			valid && (valid = true)
		} else if (required && (v == undefined || v == null || v === '') && inputCat != 'object') {
			error = 'Required'
			valid && (valid = false)
		} else {
			if (inputCat == 'number' && typeof v === 'number') {
				let min = extra['min']
				let max = extra['max']
				if (min != undefined && typeof min == 'number' && v < min) {
					error = `Should be greater than or equal to ${min}`
					valid && (valid = false)
				} else if (max != undefined && typeof max == 'number' && v > max) {
					error = `Should be less than or equal to ${max}`
					valid && (valid = false)
				} else {
					error = ''
					!valid && (valid = true)
				}
			} else if (pattern && !testRegex(pattern, v)) {
				if (!emptyString(customErrorMessage)) {
					error = customErrorMessage ?? ''
				} else if (format == 'email') {
					error = 'invalid email address'
				} else {
					error = `Should match ${pattern}`
				}
				valid && (valid = false)
			} else {
				error = ''
				!valid && (valid = true)
			}
		}
	}

	function testRegex(pattern: string, value: any): boolean {
		try {
			const regex = new RegExp(pattern)
			return regex.test(value)
		} catch (err) {
			return false
		}
	}

	function onKeyDown(e: KeyboardEvent) {
		if (
			(e.ctrlKey || e.metaKey) &&
			(e.key == 'Enter' || e.key == 'c' || e.key == 'v' || e.key == 'x')
		) {
			if (e.key == 'Enter') {
				dispatch('keydownCmdEnter')
			}
			return
		}
		e.stopPropagation()
	}

	let redraw = $state(0)
	let itemsLimit = $state(50)

	let oldValue = value

	function compareValues(value) {
		if (!deepEqual(oldValue, value)) {
			oldValue = value
			dispatch('change')
		}
	}

	let { debounced, clearDebounce } = debounce(() => compareValues(value), 50)
	let inputCat = $derived(computeInputCat(type, format, itemsType?.type, enum_, contentEncoding))
	$effect(() => {
		oneOf && untrack(() => updateOneOfSelected(oneOf))
	})
	$effect(() => {
		oneOf && value && untrack(() => onOneOfChange())
	})

	$effect.pre(() => {
		value
		let args = [inputCat, defaultValue, nullable]

		untrack(() => {
			if (deepEqual(value, lastValue)) return
			computeDefaultValue(...args)
		})
	})
	$effect.pre(() => {
		!isListJson &&
			inputCat === 'list' &&
			value != lastValue &&
			itemsType?.type &&
			!hasIsListJsonChanged &&
			untrack(() => checkArrayValueType())
	})
	$effect.pre(() => {
		defaultValue != undefined && untrack(() => handleDefaultValueChange())
	})
	$effect.pre(() => {
		;(inputCat &&
			(isObjectCat(inputCat) || isRawStringEditor(inputCat)) &&
			!oneOf &&
			untrack(() => evalValueToRaw())) ||
			value
	})

	$effect(() => {
		let args = [pattern, value, required] as const
		untrack(() => validateInput(...args))
	})
	$effect(() => {
		shouldDispatchChanges && debounced(value)
	})

	onDestroy(() => {
		clearDebounce()
	})
</script>

{#snippet variableInput()}
	{#if variableEditor}
		<div class="text-sm text-tertiary">
			{#if value && typeof value == 'string' && value?.startsWith('$var:')}
				Linked to variable <button
					class="text-blue-500 underline"
					onclick={() => variableEditor?.editVariable?.(value.slice(5))}>{value.slice(5)}</button
				>
			{/if}
		</div>
	{/if}
{/snippet}
{#snippet resourceInput()}
	{#if variableEditor}
		<div class="text-sm text-tertiary">
			{#if value && typeof value == 'string' && value?.startsWith('$res:')}
				Linked to resource <a
					target="_blank"
					href="{base}/resources#/resource/{value.slice(5)}"
					class="text-blue-500 underline"
					>{value.slice(5)} <span class="inline-block -mb-0.5"><ExternalLink size={14} /></span></a
				>
			{/if}
		</div>
	{/if}
{/snippet}
<!-- svelte-ignore a11y_autofocus -->
<div
	class={twMerge(
		'flex flex-col w-full rounded-md relative',
		minW ? 'min-w-[250px]' : '',
		diffStatus?.diff ? 'px-2' : '',
		diffStatus?.diff == 'added'
			? 'bg-green-300 dark:bg-green-800'
			: diffStatus?.diff === 'removed'
				? 'bg-red-300 dark:bg-red-800'
				: diffStatus?.diff === 'same'
					? 'bg-surface'
					: diffStatus?.diff === 'modified' || typeof diffStatus?.diff === 'object'
						? 'border-2 border-green-500 bg-surface'
						: ''
	)}
	data-schema-picker
>
	{#if diffStatus && typeof diffStatus === 'object' && diffStatus.diff !== 'same'}
		<div
			class="absolute top-0 right-2 rounded-md rounded-t-none flex flex-row overflow-hidden bg-surface"
		>
			<button
				class="p-1 bg-green-500 text-white hover:bg-green-600"
				onclick={stopPropagation(
					preventDefault(() => {
						dispatch('acceptChange', { label, nestedParent })
					})
				)}
			>
				<Check size={14} />
			</button>
			<button
				class="p-1 hover:bg-red-500 hover:text-white"
				onclick={stopPropagation(
					preventDefault(() => {
						dispatch('rejectChange', { label, nestedParent })
					})
				)}
			>
				<X size={14} />
			</button>
		</div>
	{/if}
	{#if displayHeader}
		<FieldHeader
			prettify={prettifyHeader}
			label={title && !emptyString(title) ? title : label}
			{disabled}
			{required}
			{type}
			{contentEncoding}
			{format}
			{simpleTooltip}
			{lightHeader}
			{displayType}
			labelClass={css?.label?.class}
		/>
	{/if}

	{#if description}
		<div class={twMerge('text-xs italic pb-1 text-secondary', css?.description?.class)}>
			<pre class="font-main whitespace-normal">{description}</pre>
		</div>
	{/if}

	<div class="flex space-x-1">
		{#if inputCat == 'number'}
			{#if extra['min'] != undefined && extra['max'] != undefined}
				<Range bind:value min={extra['min']} max={extra['max']} {defaultValue} />
			{:else if extra['seconds'] !== undefined}
				<SecondsInput bind:seconds={value} on:focus />
			{:else if extra?.currency}
				<CurrencyInput
					inputClasses={{
						formatted: 'px-2 w-full py-1.5 text-black dark:text-white',
						wrapper: 'w-full windmillapp',
						formattedZero: 'text-black dark:text-white'
					}}
					noColor
					bind:value
					currency={extra?.currency}
					locale={extra?.currencyLocale ?? 'en-US'}
					{disabled}
				/>
			{:else}
				<div class="relative w-full">
					<input
						{autofocus}
						onfocus={bubble('focus')}
						onblur={bubble('blur')}
						{disabled}
						type="number"
						onkeydown={() => {
							ignoreValueUndefined = true
						}}
						class={valid
							? ''
							: 'border border-red-700 border-opacity-30 focus:border-red-700 focus:border-opacity-30 bg-red-100'}
						placeholder={placeholder ?? defaultValue ?? ''}
						bind:value
						min={extra['min']}
						max={extra['max']}
					/>
				</div>
			{/if}
		{:else if inputCat == 'boolean'}
			<div class="w-full">
				<Toggle
					on:pointerdown={(e) => {
						e?.stopPropagation()
					}}
					{disabled}
					class={valid
						? ''
						: 'border border-red-700 border-opacity-30 focus:border-red-700 focus:border-opacity-30 bg-red-100'}
					bind:checked={value}
				/>
				{#if type == 'boolean' && value == undefined}
					<span>&nbsp; Not set</span>
				{/if}
			</div>
		{:else if (inputCat == 'resource-object' && format && format.split('-').length > 1 && format
				.replace('resource-', '')
				.replace('_', '')
				.toLowerCase() == 's3object') || (inputCat == 'list' && (itemsType?.resourceType === 's3_object' || itemsType?.resourceType === 's3object'))}
			<S3ArgInput
				multiple={inputCat == 'list'}
				bind:value
				{defaultValue}
				{setNewValueFromCode}
				{workspace}
				onFocus={() => dispatch('focus')}
				onBlur={() => dispatch('blur')}
				bind:editor
				{appPath}
				{computeS3ForceViewerPolicies}
			/>
		{:else if inputCat == 'list' && !isListJson}
			<div class="w-full flex gap-4">
				<div class="w-full">
					{#if Array.isArray(itemsType?.multiselect) && Array.isArray(value)}
						<div class="items-start">
							<MultiSelect
								{disabled}
								bind:value
								items={safeSelectItems(itemsType?.multiselect)}
								onOpen={() => dispatch('focus')}
								reorderable
							/>
						</div>
					{:else if enum_ && (Array.isArray(value) || value == undefined)}
						<div class="items-start">
							<MultiSelect
								{disabled}
								bind:value
								items={safeSelectItems(enum_)}
								onOpen={() => dispatch('focus')}
								reorderable
							/>
						</div>
					{:else if itemsType?.enum != undefined && Array.isArray(itemsType?.enum) && (Array.isArray(value) || value == undefined)}
						<div class="items-start">
							<MultiSelect
								{disabled}
								bind:value
								items={safeSelectItems(itemsType?.enum)}
								onOpen={() => dispatch('focus')}
								reorderable
							/>
						</div>
					{:else}
						<div class="w-full">
							{#key redraw}
								{#if Array.isArray(value)}
									{#each value ?? [] as v, i}
										{#if i < itemsLimit}
											<div class="flex max-w-md mt-1 w-full items-center">
												{#if itemsType?.type == 'number'}
													<input type="number" bind:value={value[i]} id="arg-input-number-array" />
												{:else if itemsType?.type == 'string' && itemsType?.contentEncoding == 'base64'}
													<input
														type="file"
														class="my-6"
														onchange={(x) => fileChanged(x, (val) => (value[i] = val))}
														multiple={false}
													/>
												{:else if itemsType?.type == 'object' && itemsType?.resourceType === undefined && itemsType?.properties === undefined && !(format?.startsWith('resource-') && resourceTypes?.includes(format.split('-')[1]))}
													{#await import('$lib/components/JsonEditor.svelte')}
														<Loader2 class="animate-spin" />
													{:then Module}
														<Module.default
															code={JSON.stringify(v, null, 2)}
															bind:value={value[i]}
														/>
													{/await}
												{:else if Array.isArray(itemsType?.enum)}
													<ArgEnum
														create={extra['disableCreate'] != true}
														on:focus={() => {
															dispatch('focus')
														}}
														on:blur={(e) => {
															dispatch('blur')
														}}
														{defaultValue}
														valid={valid ?? true}
														{disabled}
														{autofocus}
														bind:value={value[i]}
														enum_={itemsType?.enum ?? []}
														enumLabels={extra['enumLabels']}
													/>
												{:else if (itemsType?.type == 'resource' && itemsType?.resourceType && resourceTypes?.includes(itemsType.resourceType)) || (format?.startsWith('resource-') && resourceTypes?.includes(format.split('-')[1]))}
													{@const resourceFormat =
														itemsType?.type == 'resource' &&
														itemsType.resourceType &&
														resourceTypes.includes(itemsType.resourceType)
															? `resource-${itemsType.resourceType}`
															: format!}
													<ObjectResourceInput
														bind:value={value[i]}
														format={resourceFormat}
														defaultValue={undefined}
													/>
												{:else if itemsType?.type == 'resource'}
													{#await import('$lib/components/JsonEditor.svelte')}
														<Loader2 class="animate-spin" />
													{:then Module}
														<Module.default
															bind:editor
															on:focus={(e) => {
																dispatch('focus')
															}}
															on:blur={(e) => {
																dispatch('blur')
															}}
															code={JSON.stringify(v, null, 2)}
															bind:value={value[i]}
														/>
													{/await}
												{:else if itemsType?.type === 'object' && itemsType?.properties}
													<div class="p-8 border rounded-md w-full">
														<SchemaForm
															{onlyMaskPassword}
															{disablePortal}
															{disabled}
															{prettifyHeader}
															schema={getSchemaFromProperties(itemsType?.properties)}
															bind:args={value[i]}
														/>
													</div>
												{:else}
													<input type="text" bind:value={value[i]} id="arg-input-array" />
												{/if}
												<button
													transition:fade|local={{ duration: 100 }}
													class="rounded-full p-1 bg-surface-secondary duration-200 hover:bg-surface-hover ml-2"
													aria-label="Clear"
													onclick={() => {
														value = value.filter((_, index) => index !== i)
														redraw += 1
													}}
												>
													<X size={14} />
												</button>
											</div>
										{/if}
									{/each}
									{#if value.length > itemsLimit}
										<button onclick={() => (itemsLimit += 50)} class="text-xs py-2 text-blue-600"
											>{itemsLimit}/{value.length}: Load 50 more...</button
										>
									{/if}
								{:else if typeof value === 'string'}
									{#if value.startsWith('$res:')}
										{@render resourceInput()}
									{:else}
										<div class="text-red-500">
											Invalid string value: "{value}", expected array. Click add item to turn it
											into an array.
										</div>
									{/if}
								{/if}
							{/key}
						</div>
						<div class="flex mt-2 gap-20 items-baseline">
							<Button
								variant="border"
								color="light"
								size="xs"
								btnClasses="mt-1"
								on:click={() => {
									if (value == undefined || !Array.isArray(value)) {
										value = []
									}
									if (itemsType?.type == 'number') {
										value = value.concat(0)
									} else if (
										(itemsType?.type == 'object' &&
											!(
												format?.startsWith('resource-') &&
												resourceTypes?.includes(format.split('-')[1])
											)) ||
										(itemsType?.type == 'resource' &&
											!(
												itemsType?.resourceType && resourceTypes?.includes(itemsType?.resourceType)
											))
									) {
										value = value.concat({})
									} else {
										value = value.concat('')
									}
								}}
								id="arg-input-add-item"
								startIcon={{ icon: Plus }}
							>
								Add item
							</Button>
						</div>
					{/if}
				</div>
				<div class="mt-2 mr-4">
					<Toggle
						on:change={(e) => {
							// Once the user has changed the input type, we should not change it back automatically
							if (!hasIsListJsonChanged) {
								hasIsListJsonChanged = true
							}

							evalValueToRaw()
							isListJson = !isListJson
						}}
						checked={isListJson}
						textClass="text-secondary"
						size="xs"
						options={{ right: 'json' }}
					/>
				</div>
			</div>
		{:else if inputCat == 'dynselect'}
			<DynSelect
				name={label}
				args={otherArgs}
				{helperScript}
				bind:value
				entrypoint={format?.substring('dynselect_'.length) ?? ''}
			/>
		{:else if inputCat == 'resource-object' && resourceTypes == undefined}
			<span class="text-2xs text-tertiary">Loading resource types...</span>
		{:else if inputCat == 'resource-object' && (resourceTypes == undefined || (format && format?.split('-').length > 1 && resourceTypes.includes(format?.substring('resource-'.length))))}
			<!-- {JSON.stringify(value)} -->
			<ObjectResourceInput
				{disabled}
				{defaultValue}
				selectFirst={!noDefaultOnSelectFirst}
				{disablePortal}
				format={format ?? ''}
				bind:value
				bind:editor
				onClear={() => {
					defaultValue = null
				}}
				{showSchemaExplorer}
			/>
		{:else if inputCat == 'object' || inputCat == 'resource-object' || isListJson}
			{#if oneOf && oneOf.length >= 2}
				<div class="flex flex-col gap-2 w-full">
					{#if oneOf && oneOf.length >= 2}
						<ToggleButtonGroup
							selected={oneOfSelected}
							on:selected={({ detail }) => {
								oneOfSelected = detail
								const prevValueKeys = Object.keys(
									oneOf?.find((o) => o.title == detail)?.properties ?? {}
								)
								const toKeep = {}
								for (const key of prevValueKeys) {
									toKeep[key] = value[key]
								}
								const tagKey = oneOf.find((o) => Object.keys(o.properties ?? {}).includes('kind'))
									? 'kind'
									: 'label'
								value = { ...toKeep, [tagKey]: detail }
							}}
						>
							{#snippet children({ item })}
								{#each oneOf as obj}
									<ToggleButton value={obj.title ?? ''} label={obj.title} {item} />
								{/each}
							{/snippet}
						</ToggleButtonGroup>
						{#if oneOfSelected}
							{@const objIdx = oneOf.findIndex((o) => o.title === oneOfSelected)}
							{@const obj = oneOf[objIdx]}
							{#if obj && obj.properties && Object.keys(obj.properties).length > 0}
								{#key redraw}
									<div class="py-4 pr-2 pl-6 border rounded w-full">
										{#if orderEditable}
											<SchemaFormDnd
												{nestedClasses}
												{onlyMaskPassword}
												{disablePortal}
												{disabled}
												{prettifyHeader}
												bind:schema={
													() => ({
														properties: obj.properties ?? {},
														order: obj.order,
														$schema: '',
														required: obj.required ?? [],
														type: 'object'
													}),
													() => {
														dispatch('nestedChange')
													}
												}
												bind:args={value}
												dndType={`nested-${title}`}
												hiddenArgs={['label', 'kind']}
												on:reorder={(e) => {
													if (oneOf && oneOf[objIdx]) {
														const keys = e.detail
														oneOf[objIdx].order = keys
													}
												}}
												on:nestedChange
												{shouldDispatchChanges}
											/>
										{:else}
											<SchemaForm
												{nestedClasses}
												{onlyMaskPassword}
												{disablePortal}
												{disabled}
												{prettifyHeader}
												hiddenArgs={['label', 'kind']}
												schema={{
													properties: obj.properties,
													order: obj.order,
													$schema: '',
													required: obj.required ?? [],
													type: 'object'
												}}
												bind:args={
													() => value,
													(v) => {
														value = { ...v, [tagKey]: oneOfSelected }
													}
												}
												{shouldDispatchChanges}
												on:change={() => {
													dispatch('nestedChange')
												}}
												on:nestedChange
											/>
										{/if}
									</div>
								{/key}
							{:else if disabled}
								<textarea disabled></textarea>
							{:else}
								{#await import('$lib/components/JsonEditor.svelte')}
									<Loader2 class="animate-spin" />
								{:then Module}
									<Module.default
										bind:editor
										on:focus={(e) => {
											dispatch('focus')
										}}
										on:blur={(e) => {
											dispatch('blur')
										}}
										code={rawValue}
										on:changeValue={(e) => {
											setNewValueFromCode(e.detail)
										}}
									/>
								{/await}
							{/if}
						{/if}
					{:else if disabled}
						<textarea disabled></textarea>
					{:else}
						{#await import('$lib/components/JsonEditor.svelte')}
							<Loader2 class="animate-spin" />
						{:then Module}
							<Module.default
								bind:editor
								on:focus={(e) => {
									dispatch('focus')
								}}
								on:blur={(e) => {
									dispatch('blur')
								}}
								code={rawValue}
								on:change={(e) => {
									value = e.detail
								}}
							/>
						{/await}
					{/if}
				</div>
			{:else if properties && Object.keys(properties).length > 0 && inputCat !== 'list'}
				<div class={hideNested ? 'hidden' : 'py-4 pr-2 pl-6 border rounded-md w-full'}>
					{#if orderEditable}
						<SchemaFormDnd
							{nestedClasses}
							{onlyMaskPassword}
							{disablePortal}
							{disabled}
							{prettifyHeader}
							bind:schema={
								() => ({
									properties,
									$schema: '',
									required: nestedRequired ?? [],
									type: 'object',
									order
								}),
								(newSchema) => {
									dispatch('nestedChange')
								}
							}
							bind:args={value}
							dndType={`nested-${title}`}
							on:reorder={(e) => {
								const keys = e.detail
								order = keys
							}}
							diff={diffStatus && typeof diffStatus.diff === 'object' ? diffStatus.diff : {}}
							on:acceptChange={(e) => {
								dispatch('acceptChange', e.detail)
							}}
							on:rejectChange={(e) => {
								dispatch('rejectChange', e.detail)
							}}
							on:nestedChange
							nestedParent={{ label, nestedParent }}
							{shouldDispatchChanges}
						/>
					{:else}
						<SchemaForm
							{nestedClasses}
							{onlyMaskPassword}
							{disablePortal}
							{disabled}
							{prettifyHeader}
							schema={{
								properties,
								order,
								$schema: '',
								required: nestedRequired ?? [],
								type: 'object'
							}}
							bind:args={value}
							diff={diffStatus && typeof diffStatus.diff === 'object' ? diffStatus.diff : {}}
							nestedParent={{ label, nestedParent }}
							on:acceptChange={(e) => {
								dispatch('acceptChange', e.detail)
							}}
							on:rejectChange={(e) => {
								dispatch('rejectChange', e.detail)
							}}
							on:change={() => {
								dispatch('nestedChange')
							}}
							on:nestedChange
							{shouldDispatchChanges}
						/>
					{/if}
				</div>
			{:else if disabled}
				<textarea disabled></textarea>
			{:else}
				{#await import('$lib/components/JsonEditor.svelte')}
					<Loader2 class="animate-spin" />
				{:then Module}
					<Module.default
						bind:editor
						on:focus={(e) => {
							dispatch('focus')
						}}
						on:blur={(e) => {
							dispatch('blur')
						}}
						code={rawValue}
						on:changeValue={(e) => {
							setNewValueFromCode(e.detail)
						}}
					/>
				{/await}
			{/if}
			{#if inputCat == 'list'}
				<div class="block">
					<Toggle
						on:change={(e) => {
							isListJson = !isListJson
						}}
						checked={isListJson}
						textClass="text-secondary"
						size="xs"
						options={{ right: 'json' }}
					/>
				</div>
			{/if}
		{:else if inputCat == 'enum'}
			<div class="flex flex-row w-full gap-1">
				<ArgEnum
					create={extra['disableCreate'] != true}
					{defaultValue}
					valid={valid ?? true}
					{disabled}
					bind:value
					{enum_}
					{autofocus}
					on:focus={() => {
						dispatch('focus')
					}}
					on:blur={(e) => {
						dispatch('blur')
					}}
					enumLabels={extra['enumLabels']}
				/>
			</div>
		{:else if inputCat == 'date'}
			{#if format === 'date'}
				<DateInput {disabled} {autofocus} bind:value dateFormat={extra?.['dateFormat']} />
			{:else}
				<DateTimeInput {disabled} useDropdown {autofocus} bind:value />
			{/if}
		{:else if isRawStringEditor(inputCat)}
			{#if disabled}
				<textarea disabled></textarea>
			{:else}
				<div class="border my-1 mb-4 w-full">
					{#await import('$lib/components/SimpleEditor.svelte')}
						<Loader2 class="animate-spin" />
					{:then Module}
						<Module.default
							on:focus={(e) => {
								dispatch('focus')
							}}
							on:blur={(e) => {
								dispatch('blur')
							}}
							on:change={(e) => {
								setNewValueFromCode(e.detail?.code)
							}}
							bind:this={editor}
							lang={inputCat}
							code={typeof rawValue == 'string' ? rawValue : JSON.stringify(rawValue, null, 2)}
							autoHeight
						/>
					{/await}
				</div>
			{/if}
		{:else if inputCat == 'base64'}
			<div class="flex flex-col my-6 w-full">
				<input
					{autofocus}
					type="file"
					onchange={(x) => fileChanged(x, (val) => (value = val))}
					multiple={false}
				/>
				{#if value?.length}
					<div class="text-2xs text-tertiary mt-1"
						>File length: {value.length} base64 chars ({(value.length / 1024 / 1024).toFixed(
							2
						)}MB)</div
					>
				{/if}
			</div>
		{:else if inputCat == 'resource-string'}
			<ResourcePicker
				selectFirst={noDefaultOnSelectFirst}
				{disablePortal}
				bind:value
				initialValue={defaultValue}
				resourceType={format && format.split('-').length > 1
					? format.substring('resource-'.length)
					: undefined}
				{showSchemaExplorer}
			/>
		{:else if inputCat == 'email'}
			<input
				{autofocus}
				onfocus={bubble('focus')}
				onblur={bubble('blur')}
				{disabled}
				type="email"
				class={valid
					? ''
					: 'border border-red-700 border-opacity-30 focus:border-red-700 focus:border-opacity-3'}
				placeholder={placeholder ?? defaultValue ?? ''}
				bind:value
			/>
		{:else if inputCat == 'string'}
			<div class="flex flex-col w-full">
				<div class="flex flex-row w-full items-center justify-between relative">
					{#if password || extra?.['password'] == true}
						{#if onlyMaskPassword}
							{#if value && typeof value == 'string' && value?.startsWith('$var:')}
								<input type="text" bind:value />
							{:else}
								<Password
									{disabled}
									bind:password={value}
									placeholder={placeholder ?? defaultValue ?? ''}
								/>
							{/if}
						{:else}
							<PasswordArgInput {disabled} bind:value />
						{/if}
					{:else}
						{#key extra?.['minRows']}
							<textarea
								{autofocus}
								rows={extra?.['minRows'] ? extra['minRows']?.toString() : '1'}
								bind:this={el}
								onfocus={(e) => {
									dispatch('focus')
								}}
								onblur={(e) => {
									dispatch('blur')
								}}
								use:autosize
								onkeydown={onKeyDown}
								{disabled}
								class={twMerge(
									'w-full',
									valid
										? ''
										: 'border border-red-700 border-opacity-30 focus:border-red-700 focus:border-opacity-3'
								)}
								placeholder={placeholder ?? defaultValue ?? ''}
								bind:value
							></textarea>
						{/key}
						{#if !disabled && itemPicker && extra?.['disableVariablePicker'] != true}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<button
								class="absolute right-1 top-1 py-1 min-w-min !px-2 items-center text-gray-800 bg-surface-secondary border rounded center-center hover:bg-gray-300 transition-all cursor-pointer"
								onclick={() => {
									pickForField = label
									itemPicker?.openDrawer?.()
								}}
								title="Insert a Variable"
							>
								<DollarSign class="!text-tertiary" size={14} />
							</button>
						{/if}
					{/if}
				</div>
				{@render variableInput()}
				{@render resourceInput()}
			</div>
		{/if}
		{@render actions?.()}
	</div>

	{#if !compact || (error && error != '')}
		<div class="text-right text-xs text-red-600 dark:text-red-400">
			{#if disabled || error === ''}
				&nbsp;
			{:else}
				{error}
			{/if}
		</div>
	{:else if !noMargin}
		<div class="mb-2"></div>
	{/if}
</div>

<style>
	input::-webkit-outer-spin-button,
	input::-webkit-inner-spin-button {
		-webkit-appearance: none !important;
		margin: 0;
	}

	/* Firefox */
	input[type='number'] {
		-moz-appearance: textfield !important;
		appearance: textfield !important;
	}
</style>
