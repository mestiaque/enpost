<script setup lang="ts">
/**
 * CodeMirror 6 editor for request bodies and the response viewer. Colours come from the
 * app's CSS tokens, so it follows the theme (Dark, Light, Glass) without its own theme.
 * Ctrl+F opens CodeMirror's search panel.
 */
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { html } from "@codemirror/lang-html";
import { json } from "@codemirror/lang-json";
import { xml } from "@codemirror/lang-xml";
import { bracketMatching, foldGutter, foldKeymap, HighlightStyle, indentOnInput, syntaxHighlighting } from "@codemirror/language";
import { highlightSelectionMatches, search, searchKeymap } from "@codemirror/search";
import { Compartment, EditorState, type Extension } from "@codemirror/state";
import { drawSelection, EditorView, highlightActiveLine, highlightActiveLineGutter, keymap, lineNumbers, placeholder as cmPlaceholder } from "@codemirror/view";
import { tags as t } from "@lezer/highlight";
import { onBeforeUnmount, onMounted, ref, watch } from "vue";

export type CodeLanguage = "json" | "xml" | "html" | "text";

const props = withDefaults(defineProps<{ modelValue: string; language?: CodeLanguage; readOnly?: boolean; wrap?: boolean; placeholder?: string; label: string }>(), {
  language: "text",
  readOnly: false,
  wrap: false,
  placeholder: "",
});
const emit = defineEmits<{ "update:modelValue": [value: string] }>();

const host = ref<HTMLElement | null>(null);
let view: EditorView | null = null;
const lang = new Compartment();
const wrapping = new Compartment();
const editable = new Compartment();

const languageExt = (l: CodeLanguage): Extension => (l === "json" ? json() : l === "xml" ? xml() : l === "html" ? html() : []);

const highlight = HighlightStyle.define([
  { tag: [t.propertyName], color: "rgb(var(--accent))" },
  { tag: [t.string, t.special(t.string)], color: "#22c55e" },
  { tag: [t.number, t.bool, t.null, t.atom], color: "#f59e0b" },
  { tag: [t.keyword, t.tagName], color: "#c084fc" },
  { tag: [t.attributeName], color: "#38bdf8" },
  { tag: [t.comment], color: "rgb(var(--faint))", fontStyle: "italic" },
  { tag: [t.punctuation, t.bracket, t.separator], color: "rgb(var(--muted))" },
]);

const theme = EditorView.theme({
  "&": { height: "100%", fontSize: "var(--editor-font-size)", backgroundColor: "transparent", color: "rgb(var(--fg))" },
  ".cm-scroller": { fontFamily: "var(--editor-font)", lineHeight: "1.55" },
  ".cm-content": { caretColor: "rgb(var(--accent))", padding: "8px 0" },
  ".cm-gutters": { backgroundColor: "transparent", color: "rgb(var(--faint))", border: "none" },
  ".cm-activeLine, .cm-activeLineGutter": { backgroundColor: "rgb(var(--raised) / 0.45)" },
  "&.cm-focused": { outline: "none" },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground, ::selection": { backgroundColor: "rgb(var(--accent) / 0.28) !important" },
  ".cm-cursor": { borderLeftColor: "rgb(var(--accent))" },
  ".cm-placeholder": { color: "rgb(var(--faint))" },
  ".cm-foldPlaceholder": { backgroundColor: "rgb(var(--raised))", border: "none", color: "rgb(var(--muted))" },
  ".cm-panels": { backgroundColor: "rgb(var(--panel))", color: "rgb(var(--fg))", borderColor: "rgb(var(--line))" },
  ".cm-panels input, .cm-panels button": { fontSize: "12px" },
  ".cm-textfield": { backgroundColor: "rgb(var(--bg))", border: "1px solid rgb(var(--line))", borderRadius: "4px", color: "rgb(var(--fg))" },
  ".cm-button": { backgroundImage: "none", backgroundColor: "rgb(var(--raised))", border: "1px solid rgb(var(--line))", borderRadius: "4px", color: "rgb(var(--fg))" },
  ".cm-searchMatch": { backgroundColor: "rgb(234 179 8 / 0.3)" },
  ".cm-searchMatch-selected": { backgroundColor: "rgb(234 179 8 / 0.6)" },
});

onMounted(() => {
  if (!host.value) return;
  view = new EditorView({
    parent: host.value,
    state: EditorState.create({
      doc: props.modelValue,
      extensions: [
        lineNumbers(),
        foldGutter(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        drawSelection(),
        history(),
        indentOnInput(),
        bracketMatching(),
        highlightSelectionMatches(),
        search({ top: true }),
        syntaxHighlighting(highlight),
        // Ctrl+Enter sends the request (global shortcut): claim it so no blank line is inserted.
        keymap.of([{ key: "Mod-Enter", run: () => true }, ...defaultKeymap, ...historyKeymap, ...searchKeymap, ...foldKeymap, indentWithTab]),
        theme,
        cmPlaceholder(props.placeholder),
        EditorView.contentAttributes.of({ "aria-label": props.label }),
        lang.of(languageExt(props.language)),
        wrapping.of(props.wrap ? EditorView.lineWrapping : []),
        editable.of([EditorState.readOnly.of(props.readOnly), EditorView.editable.of(!props.readOnly)]),
        EditorView.updateListener.of((u) => {
          if (u.docChanged && !props.readOnly) emit("update:modelValue", u.state.doc.toString());
        }),
      ],
    }),
  });
});

onBeforeUnmount(() => view?.destroy());

watch(
  () => props.modelValue,
  (v) => {
    if (view && v !== view.state.doc.toString()) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: v } });
  },
);
watch(() => props.language, (l) => view?.dispatch({ effects: lang.reconfigure(languageExt(l)) }));
watch(() => props.wrap, (w) => view?.dispatch({ effects: wrapping.reconfigure(w ? EditorView.lineWrapping : []) }));
watch(
  () => props.readOnly,
  (r) => view?.dispatch({ effects: editable.reconfigure([EditorState.readOnly.of(r), EditorView.editable.of(!r)]) }),
);
</script>

<template>
  <div ref="host" class="h-full min-h-0 overflow-hidden" />
</template>
