<script lang="ts">
    import * as monaco from "monaco-editor";
    import "./polyfill";
    import Toml from "@iarna/toml";
    import { onMount } from "svelte";
    import { mode } from "mode-watcher";
    import "./editor-theme";
    import "./editor-toml";
    import { cn } from "$lib/utils";
    import {
        getMarkersFromError,
        isParseError,
        tomlCheckDebounce,
    } from "./toml-utils";
    let {
        class: className,
        default: defaultCode,
    }: { class?: string; default?: string } = $props();

    let editor: monaco.editor.IStandaloneCodeEditor;
    export const getEditor = () => {
        return editor;
    };
    export const getContent = () => {
        return editor.getValue();
    };

    let editorDiv: HTMLDivElement;
    onMount(() => {
        let model = monaco.editor.createModel(defaultCode ?? "", "toml");
        editor = monaco.editor.create(editorDiv, {
            model,
            automaticLayout: true,
            minimap: {
                enabled: false,
            },
            // lineNumbers: "off",
            lineDecorationsWidth: 0,
            lineNumbersMinChars: 1,
        });

        const disposableKeyDown = editor.onKeyDown((event) => {
            if (
                (event.ctrlKey || event.metaKey) &&
                event.keyCode == monaco.KeyCode.KeyS
            ) {
                event.preventDefault();
                window.dispatchEvent(new CustomEvent("action:apply"));
            }
        });
        const onChangeHandler = (
            _event: monaco.editor.IModelContentChangedEvent,
        ) => {
            const content = editor.getValue();
            let error: unknown;
            try {
                Toml.parse(content);
            } catch (err) {
                error = err;
            }

            const markers = isParseError(error)
                ? getMarkersFromError(error)
                : [];
            monaco.editor.setModelMarkers(editor.getModel()!, "toml", markers);
        };

        const disposable = editor.onDidChangeModelContent(
            tomlCheckDebounce(onChangeHandler),
        );
        return () => {
            disposable.dispose();
            disposableKeyDown.dispose();
        };
    });

    $effect(() => {
        if (mode.current == "dark") {
            monaco.editor.setTheme("sithra-dark");
        } else {
            monaco.editor.setTheme("sithra-light");
        }
    });

    $effect(() => {
        editor?.setValue(defaultCode ?? "");
    });
</script>

<div id="editor" bind:this={editorDiv} class={cn("", className)}></div>

<style>
</style>
