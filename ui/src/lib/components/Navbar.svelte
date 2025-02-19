<script>
	import { DarkMode, Navbar, NavBrand } from 'flowbite-svelte';
	import logo from '$lib/assets/logo.png';
</script>

<Navbar>
	<div class="flex w-full justify-between">
		<div class="flex justify-center">
			<NavBrand>
				<img src={logo} class="w-[50px] dark:invert" alt="gates logo" />
			</NavBrand>
		</div>
		<div class="flex flex-1 place-items-center justify-end">
			<DarkMode class="cursor-pointer" />
		</div>
	</div>
</Navbar>
