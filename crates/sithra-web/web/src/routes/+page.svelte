<script lang="ts">
    import { title } from "$lib/state.svelte";
    $effect(() => {
        title.set("Sithra");
    });
</script>

<div class="h-full w-full flex items-center justify-center flex-col">
    <!-- <img src="/icon.png" alt="Sithra Logo" class="w-12" /> -->
    <!-- <h1 class="text-3xl font-bold">这是主页</h1> -->
</div>
