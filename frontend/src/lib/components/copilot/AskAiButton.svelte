<script lang="ts">
	import { Button } from '$lib/components/common'
	import { WandSparkles } from 'lucide-svelte'
	import { aiChatManager } from './chat/AIChatManager.svelte'
	interface Props {
		label?: string
		initialInput?: string
		onClick?: () => void
	}

	const { label, initialInput, onClick: onClickProp }: Props = $props()

	export function onClick() {
		aiChatManager.openChat()
		if (initialInput) {
			aiChatManager.askAi(initialInput, {
				withCode: false,
				withDiff: false
			})
		}
		onClickProp?.()
	}
</script>

<Button
	iconOnly={!label}
	startIcon={{
		icon: WandSparkles
	}}
	size="xs2"
	btnClasses="!text-violet-800 dark:!text-violet-400 border border-gray-200 dark:border-gray-600 !bg-surface"
	on:click={onClick}
>
	{label}
</Button>
