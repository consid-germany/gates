<script>
	import { DarkMode, Navbar, NavBrand, NavUl } from "flowbite-svelte";
	import logo from "$lib/assets/logo.png";
</script>

<Navbar>
	<div class="w-full flex justify-between">
		<div class="flex-1 flex">
			<NavUl>
			</NavUl>
		</div>
		<div class="flex justify-center">
			<NavBrand>
				<img src={logo} class="w-[50px] dark:invert" alt="gates logo" />
			</NavBrand>
		</div>
		<div class="flex-1 flex place-items-center justify-end">
			<DarkMode />
		</div>
	</div>
</Navbar>
