<script lang="ts">
	import { clone, pluralize } from '$lib/utils'
	import { Pyramid } from 'lucide-svelte'
	import { twMerge } from 'tailwind-merge'
	import { Popover } from '../meltComponents'
	import S3FilePicker from '../S3FilePicker.svelte'
	import {
		assetsEq,
		formatAssetAccessType,
		formatAssetKind,
		getAccessType,
		type Asset,
		type AssetWithAltAccessType
	} from './lib'
	import DbManagerDrawer from '../DBManagerDrawer.svelte'
	import { untrack } from 'svelte'
	import { ResourceService } from '$lib/gen'
	import { workspaceStore } from '$lib/stores'
	import Tooltip from '../meltComponents/Tooltip.svelte'
	import Tooltip2 from '../Tooltip.svelte'
	import ResourceEditorDrawer from '../ResourceEditorDrawer.svelte'
	import type { Placement } from '@floating-ui/core'
	import AssetButtons from './AssetButtons.svelte'
	import ToggleButtonGroup from '../common/toggleButton-v2/ToggleButtonGroup.svelte'
	import ToggleButton from '../common/toggleButton-v2/ToggleButton.svelte'

	let {
		assets,
		enableChangeAnimation = true,
		size = 'xs',
		noBtnText = false,
		popoverPlacement = 'bottom-end',
		disableLiTooltip = false,
		onHoverLi,
		liSubtitle
	}: {
		assets: AssetWithAltAccessType[]
		enableChangeAnimation?: boolean
		size?: 'xs' | '3xs'
		noBtnText?: boolean
		popoverPlacement?: Placement
		disableLiTooltip?: boolean
		onHoverLi?: (asset: Asset, eventType: 'enter' | 'leave') => void
		liSubtitle?: (asset: Asset) => string
	} = $props()

	let prevAssets = $state<typeof assets>([])
	let blueBgDiv: HTMLDivElement | undefined = $state()

	let s3FilePicker: S3FilePicker | undefined = $state()
	let dbManagerDrawer: DbManagerDrawer | undefined = $state()
	let resourceEditorDrawer: ResourceEditorDrawer | undefined = $state()
	let isOpen = $state(false)
	let resourceDataCache: Record<string, string | undefined> = $state({})

	$effect(() => {
		if (!enableChangeAnimation) {
			if (blueBgDiv) {
				blueBgDiv.classList.remove('animate-fade-out')
			}
		}
	})

	$effect(() => {
		assets
		untrack(() => {
			if (assetsEq(assets, prevAssets)) return
			prevAssets = clone(assets)

			// Replay animation
			if (blueBgDiv && enableChangeAnimation) {
				blueBgDiv.classList.add('animate-fade-out')
				blueBgDiv.style.animation = 'none'
				blueBgDiv.offsetHeight /* trigger reflow */
				blueBgDiv.style.animation = ''
			}

			for (const asset of assets) {
				if (asset.kind !== 'resource' || asset.path in resourceDataCache) continue
				ResourceService.getResource({ path: asset.path, workspace: $workspaceStore! })
					.then((resource) => (resourceDataCache[asset.path] = resource.resource_type))
					.catch((err) => (resourceDataCache[asset.path] = undefined))
			}
		})
	})
</script>

<Popover
	floatingConfig={{ strategy: 'absolute', placement: popoverPlacement }}
	usePointerDownOutside
	closeOnOtherPopoverOpen
	bind:isOpen
	escapeBehavior="ignore"
>
	<svelte:fragment slot="trigger">
		<div
			class={twMerge(
				size === '3xs' ? 'h-[1.6rem]' : 'py-1.5',
				'text-xs flex items-center gap-1.5 px-2 rounded-md relative',
				'border border-tertiary/30',
				'bg-surface hover:bg-surface-hover active:bg-surface',
				'transition-all hover:text-primary cursor-pointer'
			)}
		>
			<div
				bind:this={blueBgDiv}
				class="absolute pointer-events-none bg-slate-300 dark:bg-[#576278] inset-0 rounded-md opacity-0"
			></div>
			<Pyramid size={size === '3xs' ? 13 : 16} class="z-10" />
			<span
				class={twMerge('z-10 font-normal', size === '3xs' ? 'text-3xs mt-[0.08rem]' : 'text-xs')}
			>
				{noBtnText ? assets.length : pluralize(assets.length, 'asset')}
			</span>
		</div>
	</svelte:fragment>
	<svelte:fragment slot="content">
		<ul class="divide-y rounded-md">
			{#each assets as asset}
				<li
					class="text-sm px-3 h-12 flex gap-3 items-center hover:bg-surface-hover/25"
					onmouseenter={() => onHoverLi?.(asset, 'enter')}
					onmouseleave={() => onHoverLi?.(asset, 'leave')}
				>
					<Popover
						contentClasses="py-2 px-4 flex flex-col gap-2"
						disablePopup={!!asset.access_type}
					>
						<svelte:fragment slot="trigger">
							<div
								class={twMerge(
									'text-xs font-normal border text-tertiary w-10 p-1 text-center rounded-md',
									!asset.access_type && !asset.alt_access_type
										? 'text-orange-500 !border-orange-500'
										: '',
									!asset.access_type ? 'hover:bg-surface active:opacity-80' : ''
								)}
							>
								{formatAssetAccessType(getAccessType(asset))}
							</div>
						</svelte:fragment>
						<svelte:fragment slot="content">
							{#if !asset.access_type}
								<span class="text-sm text-tertiary leading-4">
									Could not infer automatically <br />
									<span class="text-xs">Please select manually </span>
								</span>
								<div class="flex items-center gap-2">
									<ToggleButtonGroup bind:selected={asset.alt_access_type} class="max-w-fit">
										{#snippet children({ item })}
											<ToggleButton value="r" label="Read" {item} />
											<ToggleButton value="w" label="Write" {item} />
											<ToggleButton value="rw" label="Read/Write" {item} />
										{/snippet}
									</ToggleButtonGroup>

									<Tooltip2>
										This is used to determine if the asset should be displayed as an input or an
										output node in the flow editor
									</Tooltip2>
								</div>
							{/if}
						</svelte:fragment>
					</Popover>
					<div class="flex flex-col flex-1">
						<Tooltip class="select-none max-w-48 truncate" disablePopup={disableLiTooltip}>
							{asset.path}
							<svelte:fragment slot="text">
								{asset.path}
							</svelte:fragment>
						</Tooltip>
						<span class="text-xs text-tertiary select-none">
							{liSubtitle?.(asset) ??
								formatAssetKind({
									...asset,
									...(asset.kind === 'resource'
										? { metadata: { resource_type: resourceDataCache[asset.path] } }
										: {})
								})}
						</span>
					</div>

					<AssetButtons
						onClick={() => (isOpen = false)}
						{asset}
						{resourceDataCache}
						{dbManagerDrawer}
						{resourceEditorDrawer}
						{s3FilePicker}
					/>
				</li>
			{/each}
		</ul>
	</svelte:fragment>
</Popover>
<S3FilePicker bind:this={s3FilePicker} readOnlyMode />
<DbManagerDrawer bind:this={dbManagerDrawer} />
<ResourceEditorDrawer bind:this={resourceEditorDrawer} />
